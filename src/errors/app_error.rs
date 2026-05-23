use crate::errors::error_meta::{ErrorMeta, ErrorMetadata};
use crate::errors::jwt_errors::JwtError;
use crate::errors::organization_service_errors::OrgServiceError;
use crate::errors::staff_service_errors::StaffServiceError;
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
    Org(OrgServiceError),
    Staff(StaffServiceError),
    Unauthorized,
    Forbidden { permission: String },
    Database(DbErr),
    InternalServer(String),
    ActorIdNotFound,
    UserIdNotFound,
    OrganizationIdNotFound,
}

impl AppError {
    pub fn meta(&self) -> ErrorMeta {
        match self {
            AppError::User(data) => data.meta(),
            AppError::UserCredential(data) => data.meta(),
            AppError::Jwt(data) => data.meta(),
            AppError::Org(data) => data.meta(),
            AppError::Staff(data) => data.meta(),
            AppError::Database(data) => data.meta(),
            AppError::Unauthorized => ErrorMeta {
                code: "000.000.0002",
                message: "Not authorized".into(),
                http_code: HttpErrorCode::Unauthorized,
            },
            AppError::Forbidden { permission } => ErrorMeta {
                code: "000.000.0003",
                message: format!("Missing permission: {permission}"),
                http_code: HttpErrorCode::Forbidden,
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
            AppError::ActorIdNotFound => ErrorMeta {
                code: "001.000.001",
                message: "Actor id is missing inside context".into(),
                http_code: HttpErrorCode::InternalServerError,
            },
            AppError::UserIdNotFound => ErrorMeta {
                code: "001.000.002",
                message: "User id is missing inside context".into(),
                http_code: HttpErrorCode::InternalServerError,
            },
            AppError::OrganizationIdNotFound => ErrorMeta {
                code: "001.000.003",
                message: "Organization id is missing inside context".into(),
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
    Forbidden,
}
impl HttpErrorCode {
    pub fn as_status(&self) -> axum::http::StatusCode {
        use axum::http::StatusCode;

        match self {
            Self::NotFound => StatusCode::NOT_FOUND,

            Self::Conflict => StatusCode::CONFLICT,

            Self::Unauthorized => StatusCode::UNAUTHORIZED,

            Self::Forbidden => StatusCode::FORBIDDEN,

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
