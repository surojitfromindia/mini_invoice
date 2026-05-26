use sea_orm::ConnectionTrait;

use crate::api::dto::item_dto::CreateItemResolutionInput;
use crate::entity::{OrganizationPrimaryId, UnitPrimaryId};
use crate::errors::app_error::AppError;
use crate::resolver::public_id_resolver::PublicIdResolver;
use crate::service::item_service::CreateItemInput;

pub struct ItemPayloadResolver;

impl ItemPayloadResolver {
    // Item creation accepts unit public ids at the API edge, then resolves
    // them once so the service only receives internal foreign keys.
    pub async fn create_item(
        db_transaction: &impl ConnectionTrait,
        organization_id: OrganizationPrimaryId,
        payload: CreateItemResolutionInput,
    ) -> Result<CreateItemInput, AppError> {
        let unit_ids = PublicIdResolver::unit_ids(
            db_transaction,
            organization_id,
            &[
                payload.base_unit_public_id.clone(),
                payload.purchase_unit_public_id.clone(),
                payload.sales_unit_public_id.clone(),
            ],
        )
        .await?;

        let [base_unit_id, purchase_unit_id, sales_unit_id]: [UnitPrimaryId; 3] = unit_ids
            .try_into()
            .map_err(|_| AppError::InternalServer("Failed to resolve item units".into()))?;

        Ok(CreateItemInput {
            sku: payload.sku,
            barcode: payload.barcode,
            name_primary: payload.name_primary,
            name_secondary: payload.name_secondary,
            description: payload.description,
            item_type: payload.item_type.into_service_input(),
            base_unit_id,
            purchase_unit_id,
            sales_unit_id,
            default_purchase_price: payload.default_purchase_price,
            default_sales_price: payload.default_sales_price,
            track_inventory: payload.track_inventory,
            allow_negative_stock: payload.allow_negative_stock,
            reorder_level: payload.reorder_level,
            is_active: payload.is_active,
        })
    }
}
