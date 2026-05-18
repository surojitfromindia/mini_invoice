use sea_orm::entity::prelude::*;
use crate::entity::{ActorPrimaryId, OrganizationPrimaryId, PublicId, UserPrimaryId};



#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "staff_status")]
pub enum StaffStatus {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "deleted")]
    Deleted,
}

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "staffs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique_key = "org_staff")]
    pub user_id: UserPrimaryId,
    #[sea_orm(unique_key = "org_staff")]
    pub organization_id: OrganizationPrimaryId,
    #[sea_orm(unique)]
    pub public_id: PublicId,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub status : StaffStatus,
    pub created_by_actor_id: ActorPrimaryId,
    pub updated_by_actor_id: Option<ActorPrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
impl ActiveModelBehavior for ActiveModel {}
