use sea_orm::ConnectionTrait;

use crate::entity::organization::staff_invitation_entity as StaffInvitation;
use crate::entity::{BranchPrimaryId, OrganizationPrimaryId};
use crate::errors::app_error::AppError;
use crate::errors::staff_service_errors::StaffServiceError;
use crate::service::staff_service::{
    CreateStaffInvitation, ResendStaffInvitation, RevokeStaffInvitation,
};

use super::public_id_resolver::PublicIdResolver;

pub struct ResolvedCreateStaffInvitation {
    pub invitee_email: String,
    pub invitee_first_name: String,
    pub invitee_last_name: String,
    pub invited_role_id: i32,
    pub branch_ids: Vec<BranchPrimaryId>,
}

pub struct StaffPayloadResolver;

impl StaffPayloadResolver {
    // Converts transport-facing public ids into internal foreign keys before the
    // service starts mutating data, so the service layer can stay focused on
    // business rules instead of payload lookups.
    pub async fn resolve_create_staff_invitation(
        db_transaction: &impl ConnectionTrait,
        organization_id: OrganizationPrimaryId,
        payload: CreateStaffInvitation,
    ) -> Result<ResolvedCreateStaffInvitation, AppError> {
        let branch_ids = PublicIdResolver::branch_ids_for_organization(
            db_transaction,
            organization_id,
            payload.branch_public_ids.as_deref(),
        )
        .await?;
        let role = PublicIdResolver::staff_role_for_organization(
            db_transaction,
            organization_id,
            &payload.role_public_id,
        )
        .await?;

        Ok(ResolvedCreateStaffInvitation {
            invitee_email: payload.invitee_email,
            invitee_first_name: payload.invitee_first_name,
            invitee_last_name: payload.invitee_last_name,
            invited_role_id: role.id,
            branch_ids,
        })
    }

    pub async fn resolve_resend_staff_invitation(
        db_transaction: &impl ConnectionTrait,
        organization_id: OrganizationPrimaryId,
        payload: ResendStaffInvitation,
    ) -> Result<StaffInvitation::Model, AppError> {
        Self::resolve_invitation_for_organization(
            db_transaction,
            organization_id,
            &payload.invitation_id,
        )
        .await
    }

    pub async fn resolve_revoke_staff_invitation(
        db_transaction: &impl ConnectionTrait,
        organization_id: OrganizationPrimaryId,
        payload: RevokeStaffInvitation,
    ) -> Result<StaffInvitation::Model, AppError> {
        Self::resolve_invitation_for_organization(
            db_transaction,
            organization_id,
            &payload.invitation_id,
        )
        .await
    }

    async fn resolve_invitation_for_organization(
        db_transaction: &impl ConnectionTrait,
        organization_id: OrganizationPrimaryId,
        invitation_public_id: &str,
    ) -> Result<StaffInvitation::Model, AppError> {
        let invitation =
            PublicIdResolver::staff_invitation(db_transaction, invitation_public_id).await?;
        if invitation.organization_id != organization_id {
            return Err(StaffServiceError::InvitationNotFound.into());
        }
        Ok(invitation)
    }
}
