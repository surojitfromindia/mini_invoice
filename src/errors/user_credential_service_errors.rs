use crate::errors::app_error::HttpErrorCode;
use crate::errors::error_meta::ErrorMeta;

#[derive(Debug)]
pub enum UserCredentialServiceError {
    CredentialNotFound,
}

impl UserCredentialServiceError {
    pub fn meta(&self) -> ErrorMeta {
        match self {
            UserCredentialServiceError::CredentialNotFound => ErrorMeta {
                code: "101.000.0001",
                message: "Credential not found".to_string(),
                http_code: HttpErrorCode::InternalServerError,
            },
        }
    }
}
