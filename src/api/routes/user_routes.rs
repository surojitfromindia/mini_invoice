use crate::app_state::AppState;
use crate::service::user_service::{CreateUserAccount, create_user_account};
use crate::service_cotext::ServiceContext;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use serde::Serialize;
use axum::{
    response::{IntoResponse, Response},
    http::StatusCode,
};
pub fn routes() -> Router<AppState> {
    Router::new().route("/create_account", post(create_account_handler))
}

async fn create_account_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserAccount>,
) -> ApiResponse<bool> {
    let service_context = ServiceContext { app_state: state };
    create_user_account(&service_context, payload).await;
    ApiResponse::from_success(true, "User account created")
}

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
        details: Option<impl Into<String>>,
    ) -> Self {
        Self {
            success: false,
            data: None,
            message: message.into(),
            error: Some(ApiErrorBody {
                code: code.into(),
                details: details.map(Into::into),
            }),
        }
    }
}


impl<T> IntoResponse for ApiResponse<T>
where
    T: serde::Serialize,
{
    fn into_response(self) -> Response {
        let status = if self.success {
            StatusCode::OK
        } else {
            StatusCode::BAD_REQUEST
        };
        (status, Json(self)).into_response()
    }
}