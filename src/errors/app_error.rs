use crate::errors::branch_service_errors::BranchServiceError;
use crate::errors::error_codes;
use crate::errors::error_meta::{ErrorMeta, ErrorMetadata};
use crate::errors::internal_error_messages;
use crate::errors::item_service_errors::ItemServiceError;
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
    Item(ItemServiceError),
    Org(OrgServiceError),
    Staff(StaffServiceError),
    Branch(BranchServiceError),
    Unauthorized,
    Forbidden { permission: String },
    BadRequest { code: &'static str, message: String },
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
            AppError::Item(data) => data.meta(),
            AppError::Org(data) => data.meta(),
            AppError::Staff(data) => data.meta(),
            AppError::Branch(data) => data.meta(),
            AppError::Database(data) => data.meta(),
            AppError::Unauthorized => ErrorMeta {
                code: error_codes::UNAUTHORIZED,
                message: "Not authorized".into(),
                http_code: HttpErrorCode::Unauthorized,
            },
            AppError::BadRequest { code, message } => ErrorMeta {
                code,
                message: message.clone(),
                http_code: HttpErrorCode::BadRequest,
            },
            AppError::Forbidden { permission } => ErrorMeta {
                code: error_codes::FORBIDDEN,
                message: format!("Missing permission: {permission}"),
                http_code: HttpErrorCode::Forbidden,
            },
            AppError::InvalidCredentials => ErrorMeta {
                code: error_codes::INVALID_CREDENTIALS,
                message: "Invalid email or password".into(),
                http_code: HttpErrorCode::Unauthorized,
            },
            AppError::InternalServer(_) => ErrorMeta {
                code: error_codes::INTERNAL_SERVER_ERROR,
                message: internal_error_messages::INTERNAL_SERVER_ERROR.into(),
                http_code: HttpErrorCode::InternalServerError,
            },
            AppError::ActorIdNotFound => ErrorMeta {
                code: error_codes::ACTOR_ID_NOT_FOUND,
                message: "Actor id is missing inside context".into(),
                http_code: HttpErrorCode::InternalServerError,
            },
            AppError::UserIdNotFound => ErrorMeta {
                code: error_codes::USER_ID_NOT_FOUND,
                message: "User id is missing inside context".into(),
                http_code: HttpErrorCode::InternalServerError,
            },
            AppError::OrganizationIdNotFound => ErrorMeta {
                code: error_codes::ORGANIZATION_ID_NOT_FOUND,
                message: "Organization id is missing inside context".into(),
                http_code: HttpErrorCode::InternalServerError,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HttpErrorCode {
    NotFound,
    BadRequest,
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

            Self::BadRequest => StatusCode::BAD_REQUEST,

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
