use crate::api::PublicContext;
use crate::api::api_response::ApiResponse;
use crate::api::dto::user_dto::{CreateUserAccountRequestDto, UserAccountCreatedResponseDto};
use crate::app_state::AppState;
use crate::errors::app_error::AppError;
use crate::service::user_service::UserService;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};

pub fn routes() -> Router<AppState> {
    Router::new().route("/create_account", post(create_account_handler))
}

async fn create_account_handler(
    PublicContext(ctx): PublicContext,
    Json(payload): Json<CreateUserAccountRequestDto>,
) -> Result<ApiResponse<UserAccountCreatedResponseDto>, AppError> {
    let email = UserService::create_user_account(&ctx, payload.into()).await?;
    Ok(ApiResponse::success(
        UserAccountCreatedResponseDto { email },
        "User account created",
        Some(StatusCode::CREATED),
    ))
}
