use sea_orm::ConnectionTrait;

use crate::entity::PrimaryId;
use crate::errors::app_error::AppError;
use crate::service::staff_service::CreateStaffInvitationInput;

use super::public_id_resolver::PublicIdResolver;

pub struct CreateStaffInvitationResolutionInput {
    pub invitee_email: String,
    pub invitee_first_name: String,
    pub invitee_last_name: String,
    pub role_public_id: String,
    pub branch_public_ids: Option<Vec<String>>,
}

pub struct StaffPayloadResolver;

impl StaffPayloadResolver {
    // Resolve a transport payload that contains multiple public ids into the
    // internal foreign keys expected by the service layer.
    pub async fn create_staff_invitation(
        db_transaction: &impl ConnectionTrait,
        organization_id: PrimaryId,
        payload: CreateStaffInvitationResolutionInput,
    ) -> Result<CreateStaffInvitationInput, AppError> {
        let branch_ids = PublicIdResolver::branch_ids(
            db_transaction,
            organization_id,
            payload.branch_public_ids.as_deref(),
        )
        .await?;
        let invited_role_id = PublicIdResolver::staff_role_id(
            db_transaction,
            organization_id,
            &payload.role_public_id,
        )
        .await?;

        Ok(CreateStaffInvitationInput {
            invitee_email: payload.invitee_email,
            invitee_first_name: payload.invitee_first_name,
            invitee_last_name: payload.invitee_last_name,
            invited_role_id,
            branch_ids,
        })
    }
}
