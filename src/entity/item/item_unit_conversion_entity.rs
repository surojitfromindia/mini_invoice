use crate::entity::{
    ActorPrimaryId, ItemPrimaryId, OrganizationPrimaryId, PublicId, UnitConversionPrimaryId,
    UnitPrimaryId,
};
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "item_unit_conversion_rounding_mode"
)]
pub enum ConversionRoundingMode {
    #[sea_orm(string_value = "none")]
    None,
    #[sea_orm(string_value = "round")]
    Round,
    #[sea_orm(string_value = "floor")]
    Floor,
    #[sea_orm(string_value = "ceil")]
    Ceil,
}

// Item-scoped unit conversions keep packaging math tied to the item instead of
// pretending every unit pair converts the same way across the organization.
#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "item_unit_conversions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: UnitConversionPrimaryId,
    #[sea_orm(unique_key = "item_unit_conversion_pair")]
    pub organization_id: OrganizationPrimaryId,
    #[sea_orm(unique)]
    pub public_id: PublicId,
    #[sea_orm(unique_key = "item_unit_conversion_pair")]
    pub item_id: ItemPrimaryId,
    #[sea_orm(unique_key = "item_unit_conversion_pair")]
    pub from_unit_id: UnitPrimaryId,
    #[sea_orm(unique_key = "item_unit_conversion_pair")]
    pub to_unit_id: UnitPrimaryId,
    pub conversion_rate: Decimal,
    pub quantity_precision: i16,
    pub rounding_mode: ConversionRoundingMode,
    pub is_active: bool,
    pub note: Option<String>,
    pub created_by_actor_id: ActorPrimaryId,
    pub updated_by_actor_id: Option<ActorPrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type ItemUnitConversionModel = Model;
pub type ItemUnitConversionEntity = Entity;
