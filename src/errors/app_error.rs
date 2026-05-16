use crate::errors::error_meta::{ErrorMeta, ErrorMetadata};
use crate::errors::jwt_errors::JwtError;
use crate::errors::user_credential_service_errors::UserCredentialServiceError;
use crate::errors::user_service_errors::UserServiceError;
use sea_orm::DbErr;
use std::fmt;

#[derive(Debug)]
#[allow(dead_code)]
pub enum AppError {
    User(UserServiceError),
    UserCredential(UserCredentialServiceError),
    InvalidCredentials,
    Jwt(JwtError),
    Unauthorized,
    Database(DbErr),
    InternalServer(String),
}

impl AppError {
    pub fn meta(&self) -> ErrorMeta {
        match self {
            AppError::User(data) => data.meta(),
            AppError::UserCredential(data) => data.meta(),
            AppError::Jwt(data) => data.meta(),
            AppError::Database(data) => data.meta(),
            AppError::Unauthorized => ErrorMeta {
                code: "000.000.0002",
                message: "Not authorized".into(),
                http_code: HttpErrorCode::Unauthorized,
            },
            AppError::InvalidCredentials => ErrorMeta {
                code: "000.000.0001",
                message: "Invalid email or password".into(),
                http_code: HttpErrorCode::Unauthorized,
            },
            AppError::InternalServer(error_message) => ErrorMeta {
                code: "100.000.000",
                message: error_message.into(),
                http_code: HttpErrorCode::InternalServerError,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HttpErrorCode {
    NotFound,
    Conflict,
    InternalServerError,
    Unauthorized,
}
impl HttpErrorCode {
    pub fn as_status(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::NotFound => StatusCode::NOT_FOUND,

            Self::Conflict => StatusCode::CONFLICT,

            Self::Unauthorized => StatusCode::UNAUTHORIZED,

            Self::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl std::error::Error for AppError {}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.meta().message)
    }
}
