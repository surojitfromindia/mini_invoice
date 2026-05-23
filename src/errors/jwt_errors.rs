use crate::errors::app_error::{AppError, HttpErrorCode};
use crate::errors::error_codes;
use crate::errors::error_meta::{ErrorMeta, ErrorMetadata};

#[derive(Debug)]
pub enum JwtError {
    InvalidToken,
    InvalidTokenType,
    CannotGenerateToken,
}

impl From<JwtError> for AppError {
    fn from(err: JwtError) -> Self {
        AppError::Jwt(err)
    }
}

impl ErrorMetadata for JwtError {
    fn meta(&self) -> ErrorMeta {
        match self {
            JwtError::InvalidToken => ErrorMeta::new(
                error_codes::JWT_INVALID_TOKEN,
                "Invalid token",
                HttpErrorCode::Unauthorized,
            ),
            JwtError::InvalidTokenType => ErrorMeta::new(
                error_codes::JWT_INVALID_TOKEN_TYPE,
                "Invalid token type",
                HttpErrorCode::Unauthorized,
            ),

            JwtError::CannotGenerateToken => ErrorMeta::new(
                error_codes::JWT_CANNOT_GENERATE_TOKEN,
                "Cannot generate token",
                HttpErrorCode::InternalServerError,
            ),
        }
    }
}
