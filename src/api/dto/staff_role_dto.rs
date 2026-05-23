use serde::Deserialize;

use crate::service::staff_role_service::CreateStaffRoleInput;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateStaffRoleRequestDto {
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub permission_codes: Vec<String>,
}

impl From<CreateStaffRoleRequestDto> for CreateStaffRoleInput {
    fn from(value: CreateStaffRoleRequestDto) -> Self {
        Self {
            name_primary: value.name_primary,
            name_secondary: value.name_secondary,
            permission_codes: value.permission_codes,
        }
    }
}
