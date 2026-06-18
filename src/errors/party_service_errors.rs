use crate::errors::app_error::{AppError, HttpErrorCode};
use crate::errors::error_codes;
use crate::errors::error_meta::{ErrorMeta, ErrorMetadata};

#[derive(Debug)]
pub enum PartyServiceError {
    NotFound,
}

impl From<PartyServiceError> for AppError {
    fn from(err: PartyServiceError) -> Self {
        AppError::Party(err)
    }
}

impl ErrorMetadata for PartyServiceError {
    fn meta(&self) -> ErrorMeta {
        match self {
            Self::NotFound => ErrorMeta::new(
                error_codes::PARTY_NOT_FOUND,
                "Party not found",
                HttpErrorCode::NotFound,
            ),
        }
    }
}
