use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::common_dto::IntoServiceInput;
use crate::service::auth_service::AuthTokens;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
pub struct LoginRequestDto {
    pub email: String,
    pub password: String,
}

impl IntoServiceInput<(String, String)> for LoginRequestDto {
    fn into_service_input(self) -> (String, String) {
        (self.email, self.password)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenRequestDto {
    pub refresh_token: String,
}

impl IntoServiceInput<String> for RefreshTokenRequestDto {
    fn into_service_input(self) -> String {
        self.refresh_token
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuthTokensResponseDto {
    pub access_token: String,
    pub refresh_token: String,
}

impl AuthTokensResponseDto {
    pub fn from_service_output(tokens: AuthTokens) -> Self {
        Self {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_tokens_response_serializes_camel_case_keys() {
        let response = AuthTokensResponseDto {
            access_token: "access".to_string(),
            refresh_token: "refresh".to_string(),
        };

        let json = serde_json::to_value(response).unwrap();

        assert_eq!(
            json,
            serde_json::json!({
                "accessToken": "access",
                "refreshToken": "refresh"
            })
        );
    }

    #[test]
    fn refresh_token_request_deserializes_camel_case_keys() {
        let request: RefreshTokenRequestDto = serde_json::from_value(serde_json::json!({
            "refreshToken": "refresh"
        }))
        .unwrap();

        assert_eq!(request.refresh_token, "refresh");
    }
}
