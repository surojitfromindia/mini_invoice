use crate::entity::{PrimaryId, PublicId};
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "staff_invitation_status"
)]
pub enum StaffInvitationStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "accepted")]
    Accepted,
    #[sea_orm(string_value = "expired")]
    Expired,
    #[sea_orm(string_value = "revoked")]
    Revoked,
}

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "staff_invitations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: PrimaryId,
    #[sea_orm(unique)]
    pub public_id: PublicId,
    pub organization_id: PrimaryId,
    pub invitee_email: String,
    pub invitee_first_name: String,
    pub invitee_last_name: String,
    pub invited_role_id: PrimaryId,
    pub invitation_token_hash: String,
    #[sea_orm(unique)]
    pub invitation_token_id: String,
    pub token_expires_at: DateTimeUtc,
    pub accepted_at: Option<DateTimeUtc>,
    pub status: StaffInvitationStatus,
    pub created_by_actor_id: PrimaryId,
    pub updated_by_actor_id: Option<PrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type StaffInvitationModel = Model;
pub type StaffInvitationEntity = Entity;
