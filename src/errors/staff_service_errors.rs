use crate::errors::app_error::{AppError, HttpErrorCode};
use crate::errors::error_codes;
use crate::errors::error_meta::{ErrorMeta, ErrorMetadata};

#[derive(Debug)]
pub enum StaffServiceError {
    NotFound,
    RoleNotFound,
    InvalidPermission,
    InvitationNotFound,
    InvitationExpired,
    InvitationAlreadyUsed,
    SystemRoleProtected,
}

impl From<StaffServiceError> for AppError {
    fn from(err: StaffServiceError) -> Self {
        AppError::Staff(err)
    }
}

impl ErrorMetadata for StaffServiceError {
    fn meta(&self) -> ErrorMeta {
        match self {
            Self::NotFound => ErrorMeta::new(
                error_codes::STAFF_NOT_FOUND,
                "Staff not found",
                HttpErrorCode::NotFound,
            ),
            Self::RoleNotFound => ErrorMeta::new(
                error_codes::STAFF_ROLE_NOT_FOUND,
                "Staff role not found",
                HttpErrorCode::NotFound,
            ),
            Self::InvalidPermission => ErrorMeta::new(
                error_codes::STAFF_INVALID_PERMISSION,
                "Invalid staff permission",
                HttpErrorCode::BadRequest,
            ),
            Self::InvitationNotFound => ErrorMeta::new(
                error_codes::STAFF_INVITATION_NOT_FOUND,
                "Staff invitation not found",
                HttpErrorCode::NotFound,
            ),
            Self::InvitationExpired => ErrorMeta::new(
                error_codes::STAFF_INVITATION_EXPIRED,
                "Staff invitation has expired",
                HttpErrorCode::Conflict,
            ),
            Self::InvitationAlreadyUsed => ErrorMeta::new(
                error_codes::STAFF_INVITATION_ALREADY_USED,
                "Staff invitation is not valid anymore",
                HttpErrorCode::Conflict,
            ),
            Self::SystemRoleProtected => ErrorMeta::new(
                error_codes::STAFF_ROLE_SYSTEM_PROTECTED,
                "System staff role cannot be deleted",
                HttpErrorCode::Conflict,
            ),
        }
    }
}
