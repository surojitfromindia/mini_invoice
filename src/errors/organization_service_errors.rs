use crate::errors::app_error::{AppError, HttpErrorCode};
use crate::errors::error_meta::{ErrorMeta, ErrorMetadata};

#[derive(Debug)]
pub enum OrgServiceError {
    NotFound,
}

impl From<OrgServiceError> for AppError {
    fn from(err: OrgServiceError) -> Self {
        AppError::Org(err)
    }
}

impl ErrorMetadata for OrgServiceError {
    fn meta(&self) -> ErrorMeta {
        match self {
            Self::NotFound => {
                ErrorMeta::new("102.000.0002", "Organization not found", HttpErrorCode::NotFound)
            }
        }
    }
}
