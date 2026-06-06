use crate::errors::app_error::AppError;
use aide::operation::OperationOutput;
use aide::{generate::GenContext, openapi::Operation};
use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,

    #[serde(skip)]
    #[schemars(skip)]
    pub status: StatusCode,
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
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

impl<T: JsonSchema> OperationOutput for ApiResponse<T> {
    type Inner = Self;

    fn operation_response(
        ctx: &mut GenContext,
        operation: &mut Operation,
    ) -> Option<aide::openapi::Response> {
        <Json<Self> as OperationOutput>::operation_response(ctx, operation)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let meta = self.meta();
        let status = meta.http_code.as_status();

        if status.is_server_error() {
            tracing::error!(code = meta.code, status = %status, error = ?self, "request failed");
        } else {
            tracing::warn!(code = meta.code, status = %status, error = ?self, "request failed");
        }

        let body: ApiResponse<()> = ApiResponse::error(meta.code, meta.message, None, status);

        (status, Json(body)).into_response()
    }
}

impl OperationOutput for AppError {
    type Inner = ApiResponse<()>;
}
