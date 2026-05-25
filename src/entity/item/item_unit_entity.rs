use crate::entity::{
    ActorPrimaryId, ItemPrimaryId, ItemUnitPrimaryId, UnitPrimaryId,
};
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "item_units")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: ItemUnitPrimaryId,
    #[sea_orm(unique_key = "item_unit_unique")]
    pub item_id: ItemPrimaryId,
    #[sea_orm(unique_key = "item_unit_unique")]
    pub unit_id: UnitPrimaryId,
    pub conversion_factor_to_base: Decimal,
    pub is_base_unit: bool,
    pub is_purchase_unit: bool,
    pub is_sales_unit: bool,
    pub created_by_actor_id: ActorPrimaryId,
    pub updated_by_actor_id: Option<ActorPrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type ItemUnitModel = Model;
pub type ItemUnitEntity = Entity;
