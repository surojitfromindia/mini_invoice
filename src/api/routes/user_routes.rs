use crate::api::api_response::ApiResponse;
use crate::api::dto::common_dto::IntoServiceInput;
use crate::api::dto::user_dto::{
    CreateUserAccountRequestDto, CurrentUserResponseDto, UserAccountCreatedResponseDto,
};
use crate::api::{AuthenticatedContext, PublicContext};
use crate::app_state::AppState;
use crate::errors::app_error::AppError;
use crate::service::user_service::UserService;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/create_account", post(create_account_handler))
        .route("/me", get(get_me_handler))
}

async fn create_account_handler(
    PublicContext(ctx): PublicContext,
    Json(payload): Json<CreateUserAccountRequestDto>,
) -> Result<ApiResponse<UserAccountCreatedResponseDto>, AppError> {
    // register a new user with email and password.
    let email = UserService::create_user_account(&ctx, payload.into_service_input()).await?;
    Ok(ApiResponse::success(
        UserAccountCreatedResponseDto::from_service_output(email),
        "User account created",
        Some(StatusCode::CREATED),
    ))
}

async fn get_me_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
) -> Result<ApiResponse<CurrentUserResponseDto>, AppError> {
    let user = UserService::get_current_user_profile(&ctx).await?;

    Ok(ApiResponse::success(
        CurrentUserResponseDto::from_service_output(user),
        "Current user fetched",
        None,
    ))
}
