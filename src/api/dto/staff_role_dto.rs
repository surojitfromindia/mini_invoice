use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::common_dto::{IntoServiceInput, PagePaginationQuery};
use crate::auth::permission::deserialize_permission_codes;
use crate::db::listing::PageListResult;
use crate::entity::GenericStatus;
use crate::service::staff_role_service::{
    CreateStaffRoleInput, SortDirection, StaffRoleDetail, StaffRoleListItem,
    StaffRoleListPageInput, StaffRoleSortField, UpdateStaffRoleInput,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateStaffRoleRequestDto {
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub permission_codes: Vec<String>,
}

impl IntoServiceInput<CreateStaffRoleInput> for CreateStaffRoleRequestDto {
    fn into_service_input(self) -> CreateStaffRoleInput {
        CreateStaffRoleInput {
            name_primary: self.name_primary,
            name_secondary: self.name_secondary,
            permission_codes: self.permission_codes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStaffRoleRequestDto {
    pub name_primary: Option<String>,
    pub name_secondary: Option<String>,
    pub permission_codes: Option<Vec<String>>,
    pub status: Option<StaffRoleStatusDto>,
}

impl IntoServiceInput<UpdateStaffRoleInput> for UpdateStaffRoleRequestDto {
    fn into_service_input(self) -> UpdateStaffRoleInput {
        UpdateStaffRoleInput {
            name_primary: self.name_primary,
            name_secondary: self.name_secondary,
            permission_codes: self.permission_codes,
            status: self.status.map(StaffRoleStatusDto::into_service_input),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum StaffRoleStatusDto {
    Active,
    Deleted,
}

impl StaffRoleStatusDto {
    pub fn into_service_input(self) -> GenericStatus {
        match self {
            Self::Active => GenericStatus::Active,
            Self::Deleted => GenericStatus::Deleted,
        }
    }

    pub fn from_service_output(status: GenericStatus) -> Self {
        match status {
            GenericStatus::Active => Self::Active,
            GenericStatus::Deleted => Self::Deleted,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum StaffRoleSortFieldDto {
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
pub struct StaffRoleListPageQueryDto {
    #[serde(flatten)]
    pub pagination: PagePaginationQuery,
    pub name: Option<String>,
    pub status: Option<StaffRoleStatusDto>,
    pub sort: Option<StaffRoleSortFieldDto>,
    pub direction: Option<SortDirectionDto>,
}

impl IntoServiceInput<StaffRoleListPageInput> for StaffRoleListPageQueryDto {
    fn into_service_input(self) -> StaffRoleListPageInput {
        StaffRoleListPageInput {
            page: self.pagination.page,
            per_page: self.pagination.per_page,
            name: self.name,
            status: self.status.map(StaffRoleStatusDto::into_service_input),
            sort: self.sort.map(|sort| match sort {
                StaffRoleSortFieldDto::CreatedAt => StaffRoleSortField::CreatedAt,
                StaffRoleSortFieldDto::NamePrimary => StaffRoleSortField::NamePrimary,
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
pub struct StaffRoleListItemResponseDto {
    pub public_id: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub permission_codes: Vec<String>,
    pub is_system_role: bool,
    pub status: StaffRoleStatusDto,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct StaffRoleResponseDto {
    pub public_id: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub permission_codes: Vec<String>,
    pub is_system_role: bool,
    pub status: StaffRoleStatusDto,
}

impl StaffRoleListItemResponseDto {
    pub fn from_service_output(role: StaffRoleListItem) -> Self {
        Self {
            public_id: role.public_id,
            name_primary: role.name_primary,
            name_secondary: role.name_secondary,
            permission_codes: deserialize_permission_codes(&role.permission_codes),
            is_system_role: role.is_system_role,
            status: StaffRoleStatusDto::from_service_output(role.status),
        }
    }

    pub fn page_from_service_output(
        result: PageListResult<StaffRoleListItem>,
    ) -> PageListResult<Self> {
        result.map_rows(Self::from_service_output)
    }
}

impl StaffRoleResponseDto {
    pub fn from_service_output(role: StaffRoleDetail) -> Self {
        Self {
            public_id: role.public_id,
            name_primary: role.name_primary,
            name_secondary: role.name_secondary,
            permission_codes: role.permission_codes,
            is_system_role: role.is_system_role,
            status: StaffRoleStatusDto::from_service_output(role.status),
        }
    }
}
