use crate::errors::app_error::HttpErrorCode;
use crate::errors::error_meta::ErrorMeta;

#[derive(Debug)]
pub enum UserServiceError {
    EmailAlreadyExists,
}

impl UserServiceError {
    pub fn meta(&self) -> ErrorMeta {
        match self {
            UserServiceError::EmailAlreadyExists => ErrorMeta {
                code: "100.000.0001",
                message: "Email already exists",
                http_code: HttpErrorCode::Conflict,
            },
        }
    }
}
