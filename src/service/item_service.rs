use sea_orm::entity::prelude::Decimal;
use sea_orm::{ActiveModelTrait, Set, TransactionTrait};

use crate::entity::item::item_entity::{self as Item, ItemType};
use crate::entity::{PublicId, UnitPrimaryId};
use crate::errors::app_error::AppError;
use crate::service::service_context::ServiceContext;
use crate::utils::date_helpers::DateHelper;
use crate::utils::id_generator::IdGenerator;

pub struct CreateItemInput {
    pub sku: String,
    pub barcode: Option<String>,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub description: Option<String>,
    pub item_type: ItemType,
    pub base_unit_id: UnitPrimaryId,
    pub purchase_unit_id: UnitPrimaryId,
    pub sales_unit_id: UnitPrimaryId,
    pub default_purchase_price: Decimal,
    pub default_sales_price: Decimal,
    pub track_inventory: bool,
    pub allow_negative_stock: bool,
    pub reorder_level: Option<Decimal>,
    pub is_active: Option<bool>,
}

pub struct ItemService;

impl ItemService {
    pub async fn create_item(
        ctx: &ServiceContext,
        payload: CreateItemInput,
    ) -> Result<PublicId, AppError> {
        let actor_id = ctx.get_actor_id()?;
        let organization_id = ctx.get_organization_id()?;

        let txn = ctx.app_state.primary_write_replica.begin().await?;

        let now = DateHelper::now().value();
        let item = Item::ActiveModel {
            organization_id: Set(organization_id),
            public_id: Set(IdGenerator::generate_general_id()),
            sku: Set(payload.sku),
            barcode: Set(payload.barcode),
            name_primary: Set(payload.name_primary),
            name_secondary: Set(payload.name_secondary),
            description: Set(payload.description),
            item_type: Set(payload.item_type),
            base_unit_id: Set(payload.base_unit_id),
            purchase_unit_id: Set(payload.purchase_unit_id),
            sales_unit_id: Set(payload.sales_unit_id),
            default_purchase_price: Set(payload.default_purchase_price),
            default_sales_price: Set(payload.default_sales_price),
            track_inventory: Set(payload.track_inventory),
            allow_negative_stock: Set(payload.allow_negative_stock),
            reorder_level: Set(payload.reorder_level),
            is_active: Set(payload.is_active.unwrap_or(true)),
            created_by_actor_id: Set(actor_id),
            updated_by_actor_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        txn.commit().await?;

        Ok(item.public_id)
    }
}
