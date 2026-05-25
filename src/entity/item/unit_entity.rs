use crate::entity::{ActorPrimaryId, OrganizationPrimaryId, PublicId, UnitPrimaryId};
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "organization_units")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: UnitPrimaryId,
    #[sea_orm(unique_key = "org_unit_code")]
    pub organization_id: OrganizationPrimaryId,
    #[sea_orm(unique)]
    pub public_id: PublicId,
    #[sea_orm(unique_key = "org_unit_code")]
    pub code: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub symbol: Option<String>,
    pub decimal_places: i16,
    pub is_system_unit: bool,
    pub is_active: bool,
    pub created_by_actor_id: ActorPrimaryId,
    pub updated_by_actor_id: Option<ActorPrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type UnitModel = Model;
pub type UnitEntity = Entity;
