use crate::config::settings::Settings;
use crate::errors::app_error::AppError;

use crate::utils::date_helpers::DateHelper;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

pub struct JwtHelpers<'a> {
    settings: &'a Settings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

impl<'a> JwtHelpers<'a> {
    pub fn new(settings: &'a Settings) -> Self {
        Self { settings }
    }

    pub fn generate_access_token(&self, user_public_id: &str) -> Result<String, AppError> {
        let now = DateHelper::now().value();
        let exp = DateHelper::now().add_minutes(15).value();

        let claims = AccessTokenClaims {
            sub: user_public_id.to_string(),
            iat: now.timestamp() as usize,
            exp: exp.timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.settings.jwt_access_secret.as_bytes()),
        )
        .map_err(|e| AppError::InternalServerError(e.to_string()))
    }

    pub fn verify_access_token(&self, token: &str) -> Result<AccessTokenClaims, AppError> {
        let data = decode::<AccessTokenClaims>(
            token,
            &DecodingKey::from_secret(self.settings.jwt_refresh_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| AppError::Unauthorized)?;

        Ok(data.claims)
    }
}
