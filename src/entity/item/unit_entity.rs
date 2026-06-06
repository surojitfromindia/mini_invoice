use crate::entity::{GenericStatus, PrimaryId, PublicId};
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "units")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: PrimaryId,
    #[sea_orm(unique_key = "unit_code")]
    pub organization_id: PrimaryId,
    #[sea_orm(unique)]
    pub public_id: PublicId,
    #[sea_orm(unique_key = "unit_code")]
    pub code: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub symbol: Option<String>,
    pub decimal_places: i16,
    pub is_system_unit: bool,
    pub status: GenericStatus,
    pub created_by_actor_id: PrimaryId,
    pub updated_by_actor_id: Option<PrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type UnitModel = Model;
pub type UnitEntity = Entity;
