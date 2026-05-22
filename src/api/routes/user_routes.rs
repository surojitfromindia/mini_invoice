use crate::api::{AuthenticatedContext, PublicContext};
use crate::api::api_response::ApiResponse;
use crate::app_state::AppState;
use crate::errors::app_error::AppError;
use crate::service::auth_service::{AuthService, AuthTokensResponse};
use crate::service::user_service::{CreateUserAccount, UserService};
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct RefreshTokenPayload {
    pub refresh_token: String,
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/create_account", post(create_account_handler))
        .route("/login", post(login_with_password))
        .route("/refresh_token", post(refresh_token_handler))
        .route("/logout", post(logout_handler))
}

async fn create_account_handler(
    PublicContext(ctx): PublicContext,
    Json(payload): Json<CreateUserAccount>,
) -> Result<ApiResponse<String>, AppError> {
    let email = UserService::create_user_account(&ctx, payload).await?;
    Ok(ApiResponse::success(
        email,
        "User account created",
        Some(StatusCode::CREATED),
    ))
}

async fn login_with_password(
    PublicContext(ctx): PublicContext,
    Json(payload): Json<LoginPayload>,
) -> Result<ApiResponse<AuthTokensResponse>, AppError> {
    let result = AuthService::login_with_password(&ctx, payload.email, payload.password).await?;
    Ok(ApiResponse::success(
        result,
        "User logged-in",
        Some(StatusCode::OK),
    ))
}

async fn refresh_token_handler(
    PublicContext(ctx): PublicContext,
    Json(payload): Json<RefreshTokenPayload>,
) -> Result<ApiResponse<AuthTokensResponse>, AppError> {
    let result = AuthService::refresh_tokens(&ctx, payload.refresh_token).await?;
    Ok(ApiResponse::success(
        result,
        "Tokens refreshed",
        Some(StatusCode::OK),
    ))
}

async fn logout_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
) -> Result<ApiResponse<String>, AppError> {
    AuthService::logout(&ctx).await?;
    Ok(ApiResponse::success(
        "Logged out".to_string(),
        "User logged-out",
        Some(StatusCode::OK),
    ))
}
