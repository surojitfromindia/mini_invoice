use serde::{Deserialize, Serialize};

use crate::service::auth_service::AuthTokens;

use super::common_dto::ActionStatusResponse;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct LoginRequestDto {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RefreshTokenRequestDto {
    pub refresh_token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthTokensResponseDto {
    pub access_token: String,
    pub refresh_token: String,
}

impl From<AuthTokens> for AuthTokensResponseDto {
    fn from(value: AuthTokens) -> Self {
        Self {
            access_token: value.access_token,
            refresh_token: value.refresh_token,
        }
    }
}

pub fn logout_response() -> ActionStatusResponse {
    ActionStatusResponse::new("logged_out")
}
