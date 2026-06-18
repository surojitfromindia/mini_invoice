use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::common_dto::{IntoServiceInput, PagePaginationQuery};
use crate::db::listing::PageListResult;
use crate::entity::staff::staff_entity::StaffStatus;
use crate::service::staff_service::{
    AcceptStaffInvitationInput, SortDirection, StaffDetail, StaffInvitationCreated, StaffListItem,
    StaffListPageInput, StaffSortField,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateStaffInvitationRequestDto {
    pub invitee_email: String,
    pub invitee_first_name: String,
    pub invitee_last_name: String,
    pub role_public_id: String,
    pub branch_public_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AcceptStaffInvitationRequestDto {
    pub invitation_token: String,
    pub password: String,
}

impl IntoServiceInput<AcceptStaffInvitationInput> for AcceptStaffInvitationRequestDto {
    fn into_service_input(self) -> AcceptStaffInvitationInput {
        AcceptStaffInvitationInput {
            invitation_token: self.invitation_token,
            password: self.password,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ResendStaffInvitationRequestDto {
    pub invitation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RevokeStaffInvitationRequestDto {
    pub invitation_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStaffRequestDto {
    pub name_primary: Option<String>,
    pub name_secondary: Option<String>,
    pub role_public_id: Option<String>,
    pub branch_public_ids: Option<Vec<String>>,
    pub is_default_organization: Option<bool>,
    pub status: Option<StaffStatusDto>,
}

impl UpdateStaffRequestDto {
    pub fn into_resolution_input(
        self,
    ) -> crate::resolver::staff_payload_resolver::UpdateStaffResolutionInput {
        crate::resolver::staff_payload_resolver::UpdateStaffResolutionInput {
            name_primary: self.name_primary,
            name_secondary: self.name_secondary,
            role_public_id: self.role_public_id,
            branch_public_ids: self.branch_public_ids,
            is_default_organization: self.is_default_organization,
            status: self.status.map(StaffStatusDto::into_service_input),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum StaffStatusDto {
    Active,
    Inactive,
    Deleted,
}

impl StaffStatusDto {
    pub fn into_service_input(self) -> StaffStatus {
        match self {
            Self::Active => StaffStatus::Active,
            Self::Inactive => StaffStatus::Inactive,
            Self::Deleted => StaffStatus::Deleted,
        }
    }

    pub fn from_service_output(status: StaffStatus) -> Self {
        match status {
            StaffStatus::Active => Self::Active,
            StaffStatus::Inactive => Self::Inactive,
            StaffStatus::Deleted => Self::Deleted,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum StaffSortFieldDto {
    CreatedAt,
    NamePrimary,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum SortDirectionDto {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StaffListPageQueryDto {
    #[serde(flatten)]
    pub pagination: PagePaginationQuery,
    pub name: Option<String>,
    pub status: Option<StaffStatusDto>,
    pub sort: Option<StaffSortFieldDto>,
    pub direction: Option<SortDirectionDto>,
}

impl IntoServiceInput<StaffListPageInput> for StaffListPageQueryDto {
    fn into_service_input(self) -> StaffListPageInput {
        StaffListPageInput {
            page: self.pagination.page,
            per_page: self.pagination.per_page,
            name: self.name,
            status: self.status.map(StaffStatusDto::into_service_input),
            sort: self.sort.map(|sort| match sort {
                StaffSortFieldDto::CreatedAt => StaffSortField::CreatedAt,
                StaffSortFieldDto::NamePrimary => StaffSortField::NamePrimary,
            }),
            direction: self.direction.map(|direction| match direction {
                SortDirectionDto::Asc => SortDirection::Asc,
                SortDirectionDto::Desc => SortDirection::Desc,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StaffListItemResponseDto {
    pub public_id: String,
    pub user_public_id: String,
    pub user_email: String,
    pub user_first_name: String,
    pub user_last_name: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub role_public_id: String,
    pub is_default_organization: bool,
    pub status: StaffStatusDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StaffResponseDto {
    pub public_id: String,
    pub user_public_id: String,
    pub user_email: String,
    pub user_first_name: String,
    pub user_last_name: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub role_public_id: String,
    pub branch_public_ids: Vec<String>,
    pub is_default_organization: bool,
    pub status: StaffStatusDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
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

impl StaffListItemResponseDto {
    pub fn from_service_output(staff: StaffListItem) -> Self {
        Self {
            public_id: staff.public_id,
            user_public_id: staff.user_public_id,
            user_email: staff.user_email,
            user_first_name: staff.user_first_name,
            user_last_name: staff.user_last_name,
            name_primary: staff.name_primary,
            name_secondary: staff.name_secondary,
            role_public_id: staff.role_public_id,
            is_default_organization: staff.is_default_organization,
            status: StaffStatusDto::from_service_output(staff.status),
        }
    }

    pub fn page_from_service_output(result: PageListResult<StaffListItem>) -> PageListResult<Self> {
        result.map_rows(Self::from_service_output)
    }
}

impl StaffResponseDto {
    pub fn from_service_output(staff: StaffDetail) -> Self {
        Self {
            public_id: staff.public_id,
            user_public_id: staff.user_public_id,
            user_email: staff.user_email,
            user_first_name: staff.user_first_name,
            user_last_name: staff.user_last_name,
            name_primary: staff.name_primary,
            name_secondary: staff.name_secondary,
            role_public_id: staff.role_public_id,
            branch_public_ids: staff.branch_public_ids,
            is_default_organization: staff.is_default_organization,
            status: StaffStatusDto::from_service_output(staff.status),
        }
    }
}
