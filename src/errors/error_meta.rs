use crate::errors::app_error::HttpErrorCode;

#[derive(Debug, Clone)]
pub struct ErrorMeta {
    pub code: &'static str,
    pub message: String,
    pub http_code: HttpErrorCode,
}

impl ErrorMeta {
    pub fn new(code: &'static str, message: impl Into<String>, http_code: HttpErrorCode) -> Self {
        Self {
            code,
            message: message.into(),
            http_code,
        }
    }
}

pub trait ErrorMetadata {
    fn meta(&self) -> ErrorMeta;
}
