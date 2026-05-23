use crate::api::api_response::ApiResponse;
use crate::api::dto::auth_dto::{
    AuthTokensResponseDto, LoginRequestDto, RefreshTokenRequestDto, logout_response,
};
use crate::api::{AuthenticatedContext, PublicContext};
use crate::app_state::AppState;
use crate::errors::app_error::AppError;
use crate::service::auth_service::AuthService;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/login", post(login_with_password))
        .route("/refresh_token", post(refresh_token_handler))
        .route("/logout", post(logout_handler))
}

async fn login_with_password(
    PublicContext(ctx): PublicContext,
    Json(payload): Json<LoginRequestDto>,
) -> Result<ApiResponse<AuthTokensResponseDto>, AppError> {
    let result = AuthService::login_with_password(&ctx, payload.email, payload.password).await?;
    Ok(ApiResponse::success(
        result.into(),
        "User logged-in",
        Some(StatusCode::OK),
    ))
}

async fn refresh_token_handler(
    PublicContext(ctx): PublicContext,
    Json(payload): Json<RefreshTokenRequestDto>,
) -> Result<ApiResponse<AuthTokensResponseDto>, AppError> {
    let result = AuthService::refresh_tokens(&ctx, payload.refresh_token).await?;
    Ok(ApiResponse::success(
        result.into(),
        "Tokens refreshed",
        Some(StatusCode::OK),
    ))
}

async fn logout_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
) -> Result<ApiResponse<crate::api::dto::common_dto::ActionStatusResponse>, AppError> {
    AuthService::logout(&ctx).await?;
    Ok(ApiResponse::success(
        logout_response(),
        "User logged-out",
        Some(StatusCode::OK),
    ))
}
