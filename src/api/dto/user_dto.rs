use serde::{Deserialize, Serialize};

use super::common_dto::IntoServiceInput;
use crate::service::user_service::CreateUserAccountInput;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateUserAccountRequestDto {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
}

impl IntoServiceInput<CreateUserAccountInput> for CreateUserAccountRequestDto {
    fn into_service_input(self) -> CreateUserAccountInput {
        CreateUserAccountInput {
            first_name: self.first_name,
            last_name: self.last_name,
            email: self.email,
            password: self.password,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAccountCreatedResponseDto {
    pub email: String,
}

impl UserAccountCreatedResponseDto {
    pub fn from_service_output(email: String) -> Self {
        Self { email }
    }
}
