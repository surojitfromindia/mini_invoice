use crate::api::api_response::ApiResponse;
use crate::api::dto::auth_dto::{AuthTokensResponseDto, LoginRequestDto, RefreshTokenRequestDto};
use crate::api::dto::common_dto::{ActionStatusResponse, IntoServiceInput};
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
    let (email, password) = payload.into_service_input();
    let result = AuthService::login_with_password(&ctx, email, password).await?;
    Ok(ApiResponse::success(
        AuthTokensResponseDto::from_service_output(result),
        "User logged-in",
        Some(StatusCode::OK),
    ))
}

async fn refresh_token_handler(
    PublicContext(ctx): PublicContext,
    Json(payload): Json<RefreshTokenRequestDto>,
) -> Result<ApiResponse<AuthTokensResponseDto>, AppError> {
    let refresh_token = payload.into_service_input();
    let result = AuthService::refresh_tokens(&ctx, refresh_token).await?;
    Ok(ApiResponse::success(
        AuthTokensResponseDto::from_service_output(result),
        "Tokens refreshed",
        Some(StatusCode::OK),
    ))
}

async fn logout_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
) -> Result<ApiResponse<ActionStatusResponse>, AppError> {
    AuthService::logout(&ctx).await?;
    Ok(ApiResponse::success(
        ActionStatusResponse {
            status: "logged_out".to_string(),
        },
        "User logged-out",
        Some(StatusCode::OK),
    ))
}
