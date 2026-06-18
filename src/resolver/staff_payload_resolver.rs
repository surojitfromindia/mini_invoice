use sea_orm::ConnectionTrait;

use crate::entity::PrimaryId;
use crate::entity::staff::staff_entity::StaffStatus;
use crate::errors::app_error::AppError;
use crate::service::staff_service::{CreateStaffInvitationInput, UpdateStaffInput};

use super::public_id_resolver::PublicIdResolver;

pub struct CreateStaffInvitationResolutionInput {
    pub invitee_email: String,
    pub invitee_first_name: String,
    pub invitee_last_name: String,
    pub role_public_id: String,
    pub branch_public_ids: Option<Vec<String>>,
}

pub struct UpdateStaffResolutionInput {
    pub name_primary: Option<String>,
    pub name_secondary: Option<String>,
    pub role_public_id: Option<String>,
    pub branch_public_ids: Option<Vec<String>>,
    pub is_default_organization: Option<bool>,
    pub status: Option<StaffStatus>,
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

    pub async fn update_staff(
        db_transaction: &impl ConnectionTrait,
        organization_id: PrimaryId,
        payload: UpdateStaffResolutionInput,
    ) -> Result<UpdateStaffInput, AppError> {
        let role_id = match payload.role_public_id {
            Some(role_public_id) => Some(
                PublicIdResolver::staff_role_id(db_transaction, organization_id, &role_public_id)
                    .await?,
            ),
            None => None,
        };
        let branch_ids = match payload.branch_public_ids {
            Some(branch_public_ids) => Some(
                PublicIdResolver::branch_ids(
                    db_transaction,
                    organization_id,
                    Some(&branch_public_ids),
                )
                .await?,
            ),
            None => None,
        };

        Ok(UpdateStaffInput {
            name_primary: payload.name_primary,
            name_secondary: payload.name_secondary,
            role_id,
            branch_ids,
            is_default_organization: payload.is_default_organization,
            status: payload.status,
        })
    }
}
