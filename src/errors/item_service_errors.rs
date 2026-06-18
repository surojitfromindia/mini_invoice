use crate::errors::app_error::{AppError, HttpErrorCode};
use crate::errors::error_codes;
use crate::errors::error_meta::{ErrorMeta, ErrorMetadata};

#[derive(Debug)]
pub enum ItemServiceError {
    UnitNotFound,
    NotFound,
}

impl From<ItemServiceError> for AppError {
    fn from(err: ItemServiceError) -> Self {
        AppError::Item(err)
    }
}

impl ErrorMetadata for ItemServiceError {
    fn meta(&self) -> ErrorMeta {
        match self {
            Self::UnitNotFound => ErrorMeta::new(
                error_codes::ITEM_UNIT_NOT_FOUND,
                "Unit not found",
                HttpErrorCode::NotFound,
            ),
            Self::NotFound => ErrorMeta::new(
                error_codes::ITEM_NOT_FOUND,
                "Item not found",
                HttpErrorCode::NotFound,
            ),
        }
    }
}
