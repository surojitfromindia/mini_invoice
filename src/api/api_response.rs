use crate::errors::app_error::{AppError, HttpErrorCode};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiErrorBody>,
}

#[derive(Serialize)]
pub struct ApiErrorBody {
    pub code: String,
    pub details: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn from_success(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: message.into(),
            error: None,
        }
    }

    pub fn from_error(
        code: impl Into<String>,
        message: impl Into<String>,
        details: Option<String>,
    ) -> Self {
        Self {
            success: false,
            data: None,
            message: message.into(),
            error: Some(ApiErrorBody {
                code: code.into(),
                details,
            }),
        }
    }
}
impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self)).into_response()
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let meta = self.get_meta();

        let body = ApiResponse::<()>::from_error(meta.code, meta.message, None);

        let status = match meta.http_code {
            HttpErrorCode::Conflict => StatusCode::CONFLICT,
            HttpErrorCode::NotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(body)).into_response()
    }
}
