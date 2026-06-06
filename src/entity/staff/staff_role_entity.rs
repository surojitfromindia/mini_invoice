use crate::entity::{GenericStatus, PrimaryId, PublicId};
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "staff_roles")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: PrimaryId,
    pub organization_id: PrimaryId,
    #[sea_orm(unique)]
    pub public_id: PublicId,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub permissions: String,
    pub is_system_role: bool,
    pub status: GenericStatus,
    pub created_by_actor_id: PrimaryId,
    pub updated_by_actor_id: Option<PrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type StaffRoleModel = Model;
pub type StaffRoleEntity = Entity;
