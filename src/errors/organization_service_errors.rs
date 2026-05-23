use crate::errors::app_error::{AppError, HttpErrorCode};
use crate::errors::error_codes;
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
                error_codes::ORGANIZATION_NOT_FOUND,
                "Organization not found",
                HttpErrorCode::NotFound,
            ),
            Self::BranchNotFound => ErrorMeta::new(
                error_codes::ORGANIZATION_BRANCH_NOT_FOUND,
                "Branch not found in organization",
                HttpErrorCode::NotFound,
            ),
            Self::PrimaryBranchNotConfigured => ErrorMeta::new(
                error_codes::ORGANIZATION_PRIMARY_BRANCH_NOT_CONFIGURED,
                "Primary branch is not configured for organization",
                HttpErrorCode::Conflict,
            ),
        }
    }
}
