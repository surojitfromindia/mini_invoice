use crate::errors::app_error::{AppError, HttpErrorCode};
use crate::errors::error_codes;
use crate::errors::error_meta::{ErrorMeta, ErrorMetadata};

#[derive(Debug)]
pub enum CoaServiceError {
    AccountNotFound,
    ParentAccountNotFound,
    ParentAccountInvalid,
    SystemAccountProtected,
    AccountHasChildren,
    AccountInUse,
}

impl From<CoaServiceError> for AppError {
    fn from(err: CoaServiceError) -> Self {
        AppError::Coa(err)
    }
}

impl ErrorMetadata for CoaServiceError {
    fn meta(&self) -> ErrorMeta {
        match self {
            Self::AccountNotFound => ErrorMeta::new(
                error_codes::COA_ACCOUNT_NOT_FOUND,
                "Chart of account not found",
                HttpErrorCode::NotFound,
            ),
            Self::ParentAccountNotFound => ErrorMeta::new(
                error_codes::COA_PARENT_ACCOUNT_NOT_FOUND,
                "Parent chart of account not found",
                HttpErrorCode::NotFound,
            ),
            Self::ParentAccountInvalid => ErrorMeta::new(
                error_codes::COA_PARENT_ACCOUNT_INVALID,
                "Parent chart of account must be an active non-posting account type",
                HttpErrorCode::BadRequest,
            ),
            Self::SystemAccountProtected => ErrorMeta::new(
                error_codes::COA_SYSTEM_ACCOUNT_PROTECTED,
                "System chart of account cannot be deleted",
                HttpErrorCode::Conflict,
            ),
            Self::AccountHasChildren => ErrorMeta::new(
                error_codes::COA_ACCOUNT_HAS_CHILDREN,
                "Chart of account cannot be deleted while it has active child accounts",
                HttpErrorCode::Conflict,
            ),
            Self::AccountInUse => ErrorMeta::new(
                error_codes::COA_ACCOUNT_IN_USE,
                "Chart of account cannot be deleted while it is in use",
                HttpErrorCode::Conflict,
            ),
        }
    }
}
