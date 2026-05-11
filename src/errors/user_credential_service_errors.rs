use crate::errors::app_error::{AppError, HttpErrorCode};
use crate::errors::error_meta::{ErrorMeta, ErrorMetadata};

#[derive(Debug)]
pub enum UserCredentialServiceError {
    CredentialNotFound,
}
impl From<UserCredentialServiceError> for AppError {
    fn from(err: UserCredentialServiceError) -> Self {
        AppError::UserCredential(err)
    }
}

impl ErrorMetadata for UserCredentialServiceError {
    fn meta(&self) -> ErrorMeta {
        match self {
            UserCredentialServiceError::CredentialNotFound => ErrorMeta::new(
                "101.000.0001",
                "Credential not found",
                HttpErrorCode::InternalServerError,
            ),
        }
    }
}
