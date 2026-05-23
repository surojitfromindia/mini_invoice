use crate::errors::app_error::{AppError, HttpErrorCode};
use crate::errors::error_codes;
use crate::errors::error_meta::{ErrorMeta, ErrorMetadata};

#[derive(Debug)]
pub enum UserServiceError {
    EmailAlreadyExists,
    NotFound,
}

impl From<UserServiceError> for AppError {
    fn from(err: UserServiceError) -> Self {
        AppError::User(err)
    }
}

impl ErrorMetadata for UserServiceError {
    fn meta(&self) -> ErrorMeta {
        match self {
            UserServiceError::EmailAlreadyExists => ErrorMeta::new(
                error_codes::USER_EMAIL_ALREADY_EXISTS,
                "Email already exists",
                HttpErrorCode::Conflict,
            ),
            UserServiceError::NotFound => ErrorMeta::new(
                error_codes::USER_NOT_FOUND,
                "User not found",
                HttpErrorCode::NotFound,
            ),
        }
    }
}
