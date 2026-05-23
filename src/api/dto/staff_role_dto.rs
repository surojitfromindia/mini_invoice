use serde::Deserialize;

use crate::service::staff_role_service::CreateStaffRoleInput;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateStaffRoleRequestDto {
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub permission_codes: Vec<String>,
}

impl CreateStaffRoleRequestDto {
    pub fn into_service_input(self) -> CreateStaffRoleInput {
        CreateStaffRoleInput {
            name_primary: self.name_primary,
            name_secondary: self.name_secondary,
            permission_codes: self.permission_codes,
        }
    }
}
