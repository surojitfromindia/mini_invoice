use crate::errors::app_error::{AppError, HttpErrorCode};
use crate::errors::error_codes;
use crate::errors::error_meta::{ErrorMeta, ErrorMetadata};
use crate::errors::organization_service_errors::OrgServiceError;

#[derive(Debug)]
pub enum BranchServiceError {
    NotFound,
}
impl From<BranchServiceError> for AppError {
    fn from(err: BranchServiceError) -> Self {
        AppError::Branch(err)
    }
}


impl ErrorMetadata for BranchServiceError {
    fn meta(&self) -> ErrorMeta {
        match self {
            Self::NotFound => ErrorMeta::new(
                error_codes::BRANCH_NOT_FOUND,
                "Branch not found",
                HttpErrorCode::NotFound,
            ),
            
        }
    }
}
