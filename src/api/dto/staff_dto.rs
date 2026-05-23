use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::service::staff_service::{
    AcceptStaffInvitationInput, CreateStaffInvitationInput, ResendStaffInvitationInput,
    RevokeStaffInvitationInput, StaffInvitationCreated,
};

use super::common_dto::ActionStatusResponse;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateStaffInvitationRequestDto {
    pub invitee_email: String,
    pub invitee_first_name: String,
    pub invitee_last_name: String,
    pub role_public_id: String,
    pub branch_public_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AcceptStaffInvitationRequestDto {
    pub invitation_token: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ResendStaffInvitationRequestDto {
    pub invitation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RevokeStaffInvitationRequestDto {
    pub invitation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StaffInvitationResponseDto {
    pub invitation_id: String,
    pub invitation_token: String,
    pub token_expires_at: DateTime<Utc>,
}

impl From<CreateStaffInvitationRequestDto> for CreateStaffInvitationInput {
    fn from(value: CreateStaffInvitationRequestDto) -> Self {
        Self {
            invitee_email: value.invitee_email,
            invitee_first_name: value.invitee_first_name,
            invitee_last_name: value.invitee_last_name,
            role_public_id: value.role_public_id,
            branch_public_ids: value.branch_public_ids,
        }
    }
}

impl From<AcceptStaffInvitationRequestDto> for AcceptStaffInvitationInput {
    fn from(value: AcceptStaffInvitationRequestDto) -> Self {
        Self {
            invitation_token: value.invitation_token,
            password: value.password,
        }
    }
}

impl From<ResendStaffInvitationRequestDto> for ResendStaffInvitationInput {
    fn from(value: ResendStaffInvitationRequestDto) -> Self {
        Self {
            invitation_id: value.invitation_id,
        }
    }
}

impl From<RevokeStaffInvitationRequestDto> for RevokeStaffInvitationInput {
    fn from(value: RevokeStaffInvitationRequestDto) -> Self {
        Self {
            invitation_id: value.invitation_id,
        }
    }
}

impl From<StaffInvitationCreated> for StaffInvitationResponseDto {
    fn from(value: StaffInvitationCreated) -> Self {
        Self {
            invitation_id: value.invitation_id,
            invitation_token: value.invitation_token,
            token_expires_at: value.token_expires_at,
        }
    }
}

pub fn accepted_response() -> ActionStatusResponse {
    ActionStatusResponse::new("invitation_accepted")
}

pub fn revoked_response() -> ActionStatusResponse {
    ActionStatusResponse::new("invitation_revoked")
}
