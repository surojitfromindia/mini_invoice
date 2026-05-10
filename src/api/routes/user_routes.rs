use crate::api::api_response::ApiResponse;
use crate::app_state::AppState;
use crate::errors::app_error::AppError;
use crate::service::auth_service::{AuthService, LoginResponse};
use crate::service::service_context::ServiceContext;
use crate::service::user_service::{CreateUserAccount, UserService};
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/create_account", post(create_account_handler))
        .route("/login", post(login_with_password))
}

async fn create_account_handler(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserAccount>,
) -> Result<ApiResponse<String>, AppError> {
    let service_context = ServiceContext { app_state: state };
    let email = UserService::create_user_account(&service_context, payload).await?;
    Ok(ApiResponse::from_success(email, "User account created"))
}

async fn login_with_password(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<ApiResponse<LoginResponse>, AppError> {
    let service_context = ServiceContext { app_state: state };
    let result =
        AuthService::login_with_password(&service_context, payload.email, payload.password).await?;
    Ok(ApiResponse::from_success(result, "User logged-in"))
}
