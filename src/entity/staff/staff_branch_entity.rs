use crate::entity::{ActorPrimaryId, BranchPrimaryId, StaffPrimaryId};
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "staff_branch_status"
)]
pub enum StaffBranchStatus {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "deleted")]
    Deleted,
}

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "staff_branches")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique_key = "staff_branch_unique")]
    pub staff_id: StaffPrimaryId,
    #[sea_orm(unique_key = "staff_branch_unique")]
    pub branch_id: BranchPrimaryId,
    pub status: StaffBranchStatus,
    pub created_by_actor_id: ActorPrimaryId,
    pub updated_by_actor_id: Option<ActorPrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
impl ActiveModelBehavior for ActiveModel {}
