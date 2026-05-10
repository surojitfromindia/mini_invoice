use crate::config::settings::Settings;
use crate::errors::app_error::AppError;

use crate::utils::date_helpers::DateHelper;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use crate::errors::jwt_errors::JwtError;

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

    pub fn generate_access_token(&self, user_public_id: &str) -> Result<String, JwtError> {
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
        .map_err(|e| JwtError::CannotGenerateToken)
    }


    pub fn verify_access_token(&self, token: &str) -> Result<AccessTokenClaims, JwtError> {
        let data = decode::<AccessTokenClaims>(
            token,
            &DecodingKey::from_secret(self.settings.jwt_refresh_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| JwtError::InvalidToken)?;

        Ok(data.claims)
    }
}
