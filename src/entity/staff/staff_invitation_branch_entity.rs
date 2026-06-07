use crate::entity::{GenericStatus, PrimaryId};
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "staff_invitation_branches")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: PrimaryId,
    #[sea_orm(unique_key = "invitation_branch_unique")]
    pub staff_invitation_id: PrimaryId,
    #[sea_orm(unique_key = "invitation_branch_unique")]
    pub branch_id: PrimaryId,
    pub status: GenericStatus,
    pub created_by_actor_id: PrimaryId,
    pub updated_by_actor_id: Option<PrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
impl ActiveModelBehavior for ActiveModel {}
