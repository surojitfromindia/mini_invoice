use crate::errors::app_error::HttpErrorCode;
use crate::errors::error_meta::ErrorMeta;

#[derive(Debug)]
pub enum JwtError {
    InvalidToken,
    CannotGenerateToken,
}

impl JwtError {
    pub fn meta(&self) -> ErrorMeta {
        match self {
            JwtError::InvalidToken => ErrorMeta {
                code: "001.000.0001",
                message: "Invalid token".to_string(),
                http_code: HttpErrorCode::Unauthorized,
            },
            JwtError::CannotGenerateToken => ErrorMeta {
                code : "001.000.0002",
                message: "Cannot generate token".to_string(),
                http_code: HttpErrorCode::InternalServerError,
            }
        }
    }
}