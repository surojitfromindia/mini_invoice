use serde::Deserialize;

use super::common_dto::IntoServiceInput;
use crate::service::staff_role_service::CreateStaffRoleInput;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
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
