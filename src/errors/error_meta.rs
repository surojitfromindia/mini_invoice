use crate::errors::app_error::HttpErrorCode;

pub struct ErrorMeta {
    pub code: &'static str,
    pub message: &'static str,
    pub http_code: HttpErrorCode,
}
