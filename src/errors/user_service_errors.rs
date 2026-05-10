use crate::errors::app_error::HttpErrorCode;
use crate::errors::error_meta::ErrorMeta;

#[derive(Debug)]
pub enum UserServiceError {
    EmailAlreadyExists,
    NotFound,
}

impl UserServiceError {
    pub fn meta(&self) -> ErrorMeta {
        match self {
            UserServiceError::EmailAlreadyExists => ErrorMeta {
                code: "100.000.0001",
                message: "Email already exists".to_string(),
                http_code: HttpErrorCode::Conflict,
            },
            UserServiceError::NotFound => ErrorMeta {
                code: "100.000.0002",
                message: "User not found".to_string(),
                http_code: HttpErrorCode::NotFound,
            },
        }
    }
}
