use crate::errors::app_error::HttpErrorCode;

pub struct ErrorMeta {
    pub code: &'static str,
    pub message: String,
    pub http_code: HttpErrorCode,
}
