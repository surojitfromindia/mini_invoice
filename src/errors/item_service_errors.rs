use crate::errors::app_error::{AppError, HttpErrorCode};
use crate::errors::error_codes;
use crate::errors::error_meta::{ErrorMeta, ErrorMetadata};

#[derive(Debug)]
pub enum ItemServiceError {
    UnitNotFound,
    UnitConfigurationRequired,
    DuplicateUnitConfiguration,
    InvalidBaseUnitConfiguration,
    InvalidUnitConversionFactor,
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
                "Unit not found in organization",
                HttpErrorCode::NotFound,
            ),
            Self::UnitConfigurationRequired => ErrorMeta::new(
                error_codes::ITEM_UNIT_CONFIGURATION_REQUIRED,
                "At least one unit configuration is required",
                HttpErrorCode::BadRequest,
            ),
            Self::DuplicateUnitConfiguration => ErrorMeta::new(
                error_codes::ITEM_DUPLICATE_UNIT_CONFIGURATION,
                "Unit configuration contains duplicate units",
                HttpErrorCode::BadRequest,
            ),
            Self::InvalidBaseUnitConfiguration => ErrorMeta::new(
                error_codes::ITEM_INVALID_BASE_UNIT_CONFIGURATION,
                "Exactly one base unit with conversion factor 1 is required",
                HttpErrorCode::BadRequest,
            ),
            Self::InvalidUnitConversionFactor => ErrorMeta::new(
                error_codes::ITEM_INVALID_UNIT_CONVERSION_FACTOR,
                "Unit conversion factor must be greater than zero",
                HttpErrorCode::BadRequest,
            ),
        }
    }
}
