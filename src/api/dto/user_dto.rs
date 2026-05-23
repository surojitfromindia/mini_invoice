use serde::{Deserialize, Serialize};

use crate::service::user_service::CreateUserAccountInput;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateUserAccountRequestDto {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UserAccountCreatedResponseDto {
    pub email: String,
}

impl From<CreateUserAccountRequestDto> for CreateUserAccountInput {
    fn from(value: CreateUserAccountRequestDto) -> Self {
        Self {
            first_name: value.first_name,
            last_name: value.last_name,
            email: value.email,
            password: value.password,
        }
    }
}
