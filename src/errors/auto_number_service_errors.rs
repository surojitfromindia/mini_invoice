use crate::errors::app_error::{AppError, HttpErrorCode};
use crate::errors::error_codes;
use crate::errors::error_meta::{ErrorMeta, ErrorMetadata};

#[derive(Debug)]
pub enum AutoNumberServiceError {
    SeriesNotFound,
    InvalidQuantity,
    RequestsRequired,
    InvalidSeriesKey,
    InvalidPaddingWidth,
    InvalidStartNumber,
    InvalidIncrement,
    ResetPolicyTokenMissing,
}

impl From<AutoNumberServiceError> for AppError {
    fn from(err: AutoNumberServiceError) -> Self {
        AppError::AutoNumber(err)
    }
}

impl ErrorMetadata for AutoNumberServiceError {
    fn meta(&self) -> ErrorMeta {
        match self {
            Self::SeriesNotFound => ErrorMeta::new(
                error_codes::AUTO_NUMBER_SERIES_NOT_FOUND,
                "Auto number series not found",
                HttpErrorCode::NotFound,
            ),
            Self::InvalidQuantity => ErrorMeta::new(
                error_codes::AUTO_NUMBER_INVALID_QUANTITY,
                "Auto number quantity must be greater than zero",
                HttpErrorCode::BadRequest,
            ),
            Self::RequestsRequired => ErrorMeta::new(
                error_codes::AUTO_NUMBER_INVALID_QUANTITY,
                "At least one auto number request is required",
                HttpErrorCode::BadRequest,
            ),
            Self::InvalidSeriesKey => ErrorMeta::new(
                error_codes::AUTO_NUMBER_INVALID_SERIES_KEY,
                "Auto number series key must be one of: customer, vendor, invoice, collection, payment, credit_note, sales_order, bill, vendor_credit, purchase_order",
                HttpErrorCode::BadRequest,
            ),
            Self::InvalidPaddingWidth => ErrorMeta::new(
                error_codes::AUTO_NUMBER_INVALID_CONFIG,
                "Auto number padding width must be greater than zero",
                HttpErrorCode::BadRequest,
            ),
            Self::InvalidStartNumber => ErrorMeta::new(
                error_codes::AUTO_NUMBER_INVALID_CONFIG,
                "Auto number start number must be greater than zero",
                HttpErrorCode::BadRequest,
            ),
            Self::InvalidIncrement => ErrorMeta::new(
                error_codes::AUTO_NUMBER_INVALID_CONFIG,
                "Auto number increment must be greater than zero",
                HttpErrorCode::BadRequest,
            ),
            Self::ResetPolicyTokenMissing => ErrorMeta::new(
                error_codes::AUTO_NUMBER_INVALID_CONFIG,
                "Auto number reset policy requires a matching period token",
                HttpErrorCode::BadRequest,
            ),
        }
    }
}
