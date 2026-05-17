use crate::config::settings::Settings;
use crate::errors::jwt_errors::JwtError;
use crate::utils::date_helpers::DateHelper;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

pub struct JwtHelpers<'a> {
    settings: &'a Settings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub public_id: String,
    pub exp: usize,
    pub iat: usize,
}

impl<'a> JwtHelpers<'a> {
    pub fn new(settings: &'a Settings) -> Self {
        Self { settings }
    }

    pub fn generate_access_token(&self, user_public_id: &str) -> Result<String, JwtError> {
        let now = DateHelper::now().value();
        let exp = DateHelper::now().add_minutes(15).value();

        let claims = AccessTokenClaims {
            public_id: user_public_id.to_string(),
            iat: now.timestamp() as usize,
            exp: exp.timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.settings.jwt_access_secret.as_bytes()),
        )
        .map_err(|_e| JwtError::CannotGenerateToken)
    }

    pub fn verify_access_token(&self, token: &str) -> Result<AccessTokenClaims, JwtError> {
        let data = decode::<AccessTokenClaims>(
            token,
            &DecodingKey::from_secret(self.settings.jwt_access_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| JwtError::InvalidToken)?;
        Ok(data.claims)
    }
}
