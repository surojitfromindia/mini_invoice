use crate::config::settings::Settings;
use crate::errors::jwt_errors::JwtError;
use crate::utils::date_helpers::DateHelper;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};

pub struct JwtHelpers<'a> {
    settings: &'a Settings,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub public_id: String,
    pub token_type: TokenType,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub public_id: String,
    pub token_type: TokenType,
    pub jti: String,
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
            token_type: TokenType::Access,
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

        if data.claims.token_type != TokenType::Access {
            return Err(JwtError::InvalidTokenType);
        }

        Ok(data.claims)
    }

    pub fn generate_refresh_token(&self, user_public_id: &str) -> Result<String, JwtError> {
        let now = DateHelper::now().value();
        let exp = DateHelper::now().add_days(30).value();

        let claims = RefreshTokenClaims {
            public_id: user_public_id.to_string(),
            token_type: TokenType::Refresh,
            jti: nanoid!(),
            iat: now.timestamp() as usize,
            exp: exp.timestamp() as usize,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.settings.jwt_refresh_secret.as_bytes()),
        )
        .map_err(|_| JwtError::CannotGenerateToken)
    }

    pub fn verify_refresh_token(&self, token: &str) -> Result<RefreshTokenClaims, JwtError> {
        let data = decode::<RefreshTokenClaims>(
            token,
            &DecodingKey::from_secret(self.settings.jwt_refresh_secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|_| JwtError::InvalidToken)?;

        if data.claims.token_type != TokenType::Refresh {
            return Err(JwtError::InvalidTokenType);
        }

        Ok(data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::{JwtHelpers, TokenType};
    use crate::config::settings::Settings;

    fn settings() -> Settings {
        Settings {
            database_url: "postgres://localhost/test".into(),
            login_secret_pepper: "pepper".into(),
            jwt_access_secret: "access-secret".into(),
            jwt_refresh_secret: "refresh-secret".into(),
        }
    }

    #[test]
    fn access_token_round_trip() {
        let settings = settings();
        let jwt = JwtHelpers::new(&settings);
        let token = jwt.generate_access_token("user_123").unwrap();
        let claims = jwt.verify_access_token(&token).unwrap();

        assert_eq!(claims.public_id, "user_123");
        assert_eq!(claims.token_type, TokenType::Access);
    }

    #[test]
    fn refresh_token_round_trip() {
        let settings = settings();
        let jwt = JwtHelpers::new(&settings);
        let token = jwt.generate_refresh_token("user_123").unwrap();
        let claims = jwt.verify_refresh_token(&token).unwrap();

        assert_eq!(claims.public_id, "user_123");
        assert_eq!(claims.token_type, TokenType::Refresh);
        assert!(!claims.jti.is_empty());
    }
}
