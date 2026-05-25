use crate::entity::{
    ActorPrimaryId, ItemPrimaryId, OrganizationPrimaryId, PublicId, UnitPrimaryId,
};
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "item_type")]
pub enum ItemType {
    #[sea_orm(string_value = "inventory")]
    Inventory,
    #[sea_orm(string_value = "service")]
    Service,
    #[sea_orm(string_value = "non_inventory")]
    NonInventory,
}

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "organization_items")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: ItemPrimaryId,
    #[sea_orm(unique_key = "org_item_sku", unique_key = "org_item_barcode")]
    pub organization_id: OrganizationPrimaryId,
    #[sea_orm(unique)]
    pub public_id: PublicId,
    #[sea_orm(unique_key = "org_item_sku")]
    pub sku: String,
    #[sea_orm(unique_key = "org_item_barcode")]
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
    pub is_active: bool,
    pub created_by_actor_id: ActorPrimaryId,
    pub updated_by_actor_id: Option<ActorPrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type ItemModel = Model;
pub type ItemEntity = Entity;
