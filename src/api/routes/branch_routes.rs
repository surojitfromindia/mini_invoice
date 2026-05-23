use crate::api::AuthorizedContext;
use crate::api::api_response::ApiResponse;
use crate::app_state::AppState;
use crate::auth::permission::Permission;
use crate::errors::app_error::AppError;
use crate::service::branch_service::{BranchService, CreateBranch};
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};

pub fn routes() -> Router<AppState> {
    Router::new().route("/", post(create_branch_handler))
}

async fn create_branch_handler(
    authorized_ctx: AuthorizedContext,
    Json(payload): Json<CreateBranch>,
) -> Result<ApiResponse<String>, AppError> {
    let ctx = authorized_ctx.require_all([Permission::BranchCreate])?;
    let public_id = BranchService::create_branch(&ctx, payload).await?;
    Ok(ApiResponse::success(
        public_id,
        "Branch created",
        Some(StatusCode::CREATED),
    ))
}
