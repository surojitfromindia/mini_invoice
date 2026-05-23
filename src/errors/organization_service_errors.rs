use crate::errors::app_error::{AppError, HttpErrorCode};
use crate::errors::error_meta::{ErrorMeta, ErrorMetadata};

#[derive(Debug)]
pub enum OrgServiceError {
    NotFound,
    BranchNotFound,
    PrimaryBranchNotConfigured,
}

impl From<OrgServiceError> for AppError {
    fn from(err: OrgServiceError) -> Self {
        AppError::Org(err)
    }
}

impl ErrorMetadata for OrgServiceError {
    fn meta(&self) -> ErrorMeta {
        match self {
            Self::NotFound => ErrorMeta::new(
                "102.000.0002",
                "Organization not found",
                HttpErrorCode::NotFound,
            ),
            Self::BranchNotFound => ErrorMeta::new(
                "102.000.0003",
                "Branch not found in organization",
                HttpErrorCode::NotFound,
            ),
            Self::PrimaryBranchNotConfigured => ErrorMeta::new(
                "102.000.0004",
                "Primary branch is not configured for organization",
                HttpErrorCode::Conflict,
            ),
        }
    }
}
