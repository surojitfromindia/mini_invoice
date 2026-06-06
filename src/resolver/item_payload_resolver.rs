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
        let unit_public_ids = Self::collect_unit_public_ids(
            &payload.base_unit_public_id,
            payload.purchase_unit_public_id.as_deref(),
            payload.sales_unit_public_id.as_deref(),
        );
        let unit_ids =
            PublicIdResolver::unit_ids(db_transaction, organization_id, &unit_public_ids).await?;

        // Optional purchase and sales units are resolved only when present, while
        // base unit stays mandatory for all items.
        let (base_unit_id, purchase_unit_id, sales_unit_id) = Self::map_resolved_unit_ids(
            unit_ids,
            payload.purchase_unit_public_id.is_some(),
            payload.sales_unit_public_id.is_some(),
        )?;

        Ok(CreateItemInput {
            sku: payload.sku,
            barcode: payload.barcode,
            name_primary: payload.name_primary,
            name_secondary: payload.name_secondary,
            description: payload.description,
            item_type: payload.item_type.into_service_input(),
            item_usage: payload.item_usage.into_service_input(),
            base_unit_id,
            purchase_unit_id,
            sales_unit_id,
            default_purchase_price: payload.default_purchase_price,
            default_sales_price: payload.default_sales_price,
            track_inventory: payload.track_inventory,
            allow_negative_stock: payload.allow_negative_stock,
            reorder_level: payload.reorder_level,
            status: payload.status.map(|status| status.into_service_input()),
        })
    }

    fn collect_unit_public_ids(
        base_unit_public_id: &str,
        purchase_unit_public_id: Option<&str>,
        sales_unit_public_id: Option<&str>,
    ) -> Vec<String> {
        let mut public_ids = vec![base_unit_public_id.to_string()];

        if let Some(public_id) = purchase_unit_public_id {
            public_ids.push(public_id.to_string());
        }

        if let Some(public_id) = sales_unit_public_id {
            public_ids.push(public_id.to_string());
        }

        public_ids
    }

    fn map_resolved_unit_ids(
        unit_ids: Vec<UnitPrimaryId>,
        has_purchase_unit: bool,
        has_sales_unit: bool,
    ) -> Result<(UnitPrimaryId, Option<UnitPrimaryId>, Option<UnitPrimaryId>), AppError> {
        let mut resolved_ids = unit_ids.into_iter();
        let base_unit_id = resolved_ids
            .next()
            .ok_or_else(|| AppError::InternalServer("Failed to resolve item units".into()))?;
        let purchase_unit_id =
            if has_purchase_unit {
                Some(resolved_ids.next().ok_or_else(|| {
                    AppError::InternalServer("Failed to resolve item units".into())
                })?)
            } else {
                None
            };
        let sales_unit_id =
            if has_sales_unit {
                Some(resolved_ids.next().ok_or_else(|| {
                    AppError::InternalServer("Failed to resolve item units".into())
                })?)
            } else {
                None
            };

        if resolved_ids.next().is_some() {
            return Err(AppError::InternalServer(
                "Failed to resolve item units".into(),
            ));
        }

        Ok((base_unit_id, purchase_unit_id, sales_unit_id))
    }
}

#[cfg(test)]
mod tests {
    use super::ItemPayloadResolver;

    #[test]
    fn collect_unit_public_ids_keeps_base_and_present_optional_units_in_order() {
        let public_ids =
            ItemPayloadResolver::collect_unit_public_ids("base", Some("purchase"), Some("sales"));

        assert_eq!(public_ids, vec!["base", "purchase", "sales"]);
    }

    #[test]
    fn collect_unit_public_ids_skips_missing_optional_units() {
        let public_ids = ItemPayloadResolver::collect_unit_public_ids("base", None, Some("sales"));

        assert_eq!(public_ids, vec!["base", "sales"]);
    }

    #[test]
    fn map_resolved_unit_ids_returns_nullable_optional_unit_ids() {
        let resolved = ItemPayloadResolver::map_resolved_unit_ids(vec![11, 22], false, true)
            .expect("unit ids should map");

        assert_eq!(resolved, (11, None, Some(22)));
    }
}
