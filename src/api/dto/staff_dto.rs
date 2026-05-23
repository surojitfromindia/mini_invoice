use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::service::staff_service::{
    AcceptStaffInvitationInput, CreateStaffInvitationInput, ResendStaffInvitationInput,
    RevokeStaffInvitationInput, StaffInvitationCreated,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateStaffInvitationRequestDto {
    pub invitee_email: String,
    pub invitee_first_name: String,
    pub invitee_last_name: String,
    pub role_public_id: String,
    pub branch_public_ids: Option<Vec<String>>,
}

impl CreateStaffInvitationRequestDto {
    pub fn into_service_input(self) -> CreateStaffInvitationInput {
        CreateStaffInvitationInput {
            invitee_email: self.invitee_email,
            invitee_first_name: self.invitee_first_name,
            invitee_last_name: self.invitee_last_name,
            role_public_id: self.role_public_id,
            branch_public_ids: self.branch_public_ids,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AcceptStaffInvitationRequestDto {
    pub invitation_token: String,
    pub password: String,
}

impl AcceptStaffInvitationRequestDto {
    pub fn into_service_input(self) -> AcceptStaffInvitationInput {
        AcceptStaffInvitationInput {
            invitation_token: self.invitation_token,
            password: self.password,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ResendStaffInvitationRequestDto {
    pub invitation_id: String,
}

impl ResendStaffInvitationRequestDto {
    pub fn into_service_input(self) -> ResendStaffInvitationInput {
        ResendStaffInvitationInput {
            invitation_id: self.invitation_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RevokeStaffInvitationRequestDto {
    pub invitation_id: String,
}

impl RevokeStaffInvitationRequestDto {
    pub fn into_service_input(self) -> RevokeStaffInvitationInput {
        RevokeStaffInvitationInput {
            invitation_id: self.invitation_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StaffInvitationResponseDto {
    pub invitation_id: String,
    pub invitation_token: String,
    pub token_expires_at: DateTime<Utc>,
}

impl StaffInvitationResponseDto {
    pub fn from_service_output(invitation: StaffInvitationCreated) -> Self {
        Self {
            invitation_id: invitation.invitation_id,
            invitation_token: invitation.invitation_token,
            token_expires_at: invitation.token_expires_at,
        }
    }
}
