use crate::errors::app_error::AppError;
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,

    #[serde(skip)]
    pub status: StatusCode,
}

#[derive(Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, message: impl Into<String>, status: Option<StatusCode>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
            error: None,
            status: status.unwrap_or(StatusCode::OK),
        }
    }

    pub fn error(
        code: impl Into<String>,
        message: impl Into<String>,
        details: Option<String>,
        status: StatusCode,
    ) -> Self {
        let message = message.into();
        Self {
            success: false,
            message: message.clone(),
            data: None,
            error: Some(ApiError {
                code: code.into(),
                message,
                details,
            }),
            status,
        }
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        (self.status, Json(self)).into_response()
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let meta = self.meta();

        let status = meta.http_code.as_status();

        tracing::error!(error = ?self);

        let body: ApiResponse<()> = ApiResponse::error(meta.code, meta.message, None, status);

        (meta.http_code.as_status(), Json(body)).into_response()
    }
}
