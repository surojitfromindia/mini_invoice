use crate::entity::{
    ActorPrimaryId, ItemPrimaryId, OrganizationPrimaryId, PublicId, UnitPrimaryId,
};
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "item_type")]
pub enum ItemType {
    #[sea_orm(string_value = "product")]
    Product,
    #[sea_orm(string_value = "service")]
    Service,
}

// Item usage keeps sellable and purchasable intent explicit without forcing
// downstream validation rules to infer meaning from unit or price presence.
#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "item_usage")]
pub enum ItemUsage {
    #[sea_orm(string_value = "sales")]
    Sales,
    #[sea_orm(string_value = "purchase")]
    Purchase,
    #[sea_orm(string_value = "both")]
    Both,
}

// Items use a soft-delete status so list endpoints can hide archived records
// while still allowing future recovery or auditing flows.
#[derive(Debug, Default, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "item_status")]
pub enum ItemStatus {
    #[default]
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "inactive")]
    Inactive,
    #[sea_orm(string_value = "deleted")]
    Deleted,
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
    pub item_usage: ItemUsage,
    pub base_unit_id: UnitPrimaryId,
    pub purchase_unit_id: Option<UnitPrimaryId>,
    pub sales_unit_id: Option<UnitPrimaryId>,
    pub default_purchase_price: Option<Decimal>,
    pub default_sales_price: Option<Decimal>,
    pub track_inventory: bool,
    pub allow_negative_stock: bool,
    pub reorder_level: Option<Decimal>,
    pub status: ItemStatus,
    pub created_by_actor_id: ActorPrimaryId,
    pub updated_by_actor_id: Option<ActorPrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type ItemModel = Model;
pub type ItemEntity = Entity;
