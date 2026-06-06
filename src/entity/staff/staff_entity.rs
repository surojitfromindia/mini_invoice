use crate::entity::{PrimaryId, PublicId};
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "staff_status")]
pub enum StaffStatus {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "inactive")]
    Inactive,
    #[sea_orm(string_value = "deleted")]
    Deleted,
}

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "staffs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: PrimaryId,
    #[sea_orm(unique_key = "org_staff")]
    pub user_id: PrimaryId,
    #[sea_orm(unique_key = "org_staff")]
    pub organization_id: PrimaryId,
    #[sea_orm(unique)]
    pub public_id: PublicId,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub role_id: PrimaryId,
    pub is_default_organization: bool,
    pub status: StaffStatus,
    pub created_by_actor_id: PrimaryId,
    pub updated_by_actor_id: Option<PrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
impl ActiveModelBehavior for ActiveModel {}

pub type StaffModel = Model;
pub type StaffEntity = Entity;
