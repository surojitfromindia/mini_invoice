use crate::errors::app_error::{AppError, HttpErrorCode};
use crate::errors::error_meta::{ErrorMeta, ErrorMetadata};

#[derive(Debug)]
pub enum StaffServiceError {
    NotFound,
    RoleNotFound,
    InvalidPermission,
    InvitationNotFound,
    InvitationExpired,
    InvitationAlreadyUsed,
}

impl From<StaffServiceError> for AppError {
    fn from(err: StaffServiceError) -> Self {
        AppError::Staff(err)
    }
}

impl ErrorMetadata for StaffServiceError {
    fn meta(&self) -> ErrorMeta {
        match self {
            Self::NotFound => {
                ErrorMeta::new("103.000.0002", "Staff not found", HttpErrorCode::NotFound)
            }
            Self::RoleNotFound => ErrorMeta::new(
                "103.000.0006",
                "Staff role not found",
                HttpErrorCode::NotFound,
            ),
            Self::InvalidPermission => ErrorMeta::new(
                "103.000.0007",
                "Invalid staff permission",
                HttpErrorCode::Conflict,
            ),
            Self::InvitationNotFound => ErrorMeta::new(
                "103.000.0003",
                "Staff invitation not found",
                HttpErrorCode::NotFound,
            ),
            Self::InvitationExpired => ErrorMeta::new(
                "103.000.0004",
                "Staff invitation has expired",
                HttpErrorCode::Conflict,
            ),
            Self::InvitationAlreadyUsed => ErrorMeta::new(
                "103.000.0005",
                "Staff invitation is not valid anymore",
                HttpErrorCode::Conflict,
            ),
        }
    }
}
