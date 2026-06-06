use crate::entity::{ActorPrimaryId, OrganizationPrimaryId, PublicId, StaffRolePrimaryId};
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "staff_role_status")]
pub enum StaffRoleStatus {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "deleted")]
    Deleted,
}

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "staff_roles")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: StaffRolePrimaryId,
    pub organization_id: OrganizationPrimaryId,
    #[sea_orm(unique)]
    pub public_id: PublicId,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub permissions: String,
    pub is_system_role: bool,
    pub status: StaffRoleStatus,
    pub created_by_actor_id: ActorPrimaryId,
    pub updated_by_actor_id: Option<ActorPrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type StaffRoleModel = Model;
pub type StaffRoleEntity = Entity;
