use crate::errors::app_error::{AppError, HttpErrorCode};
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
            JwtError::InvalidToken => {
                ErrorMeta::new("001.000.0001", "Invalid token", HttpErrorCode::Unauthorized)
            }
            JwtError::InvalidTokenType => ErrorMeta::new(
                "001.000.0003",
                "Invalid token type",
                HttpErrorCode::Unauthorized,
            ),

            JwtError::CannotGenerateToken => ErrorMeta::new(
                "001.000.0002",
                "Cannot generate token",
                HttpErrorCode::InternalServerError,
            ),
        }
    }
}
