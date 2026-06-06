use crate::entity::{ActorPrimaryId, PublicId};
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "organization_status"
)]
pub enum OrganizationStatus {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "deleted")]
    Deleted,
}

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "organizations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub prime_user_id: i32,
    #[sea_orm(unique)]
    pub public_id: PublicId,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub status: OrganizationStatus,
    pub created_by_actor_id: ActorPrimaryId,
    pub updated_by_actor_id: Option<ActorPrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type OrganizationModel = Model;
pub type OrganizationEntity = Entity;
