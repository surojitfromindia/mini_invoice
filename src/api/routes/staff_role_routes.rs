use crate::api::AuthorizedContext;
use crate::api::api_response::ApiResponse;
use crate::app_state::AppState;
use crate::auth::permission::Permission;
use crate::errors::app_error::AppError;
use crate::service::staff_role_service::{CreateStaffRole, StaffRoleService};
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};

pub fn routes() -> Router<AppState> {
    Router::new().route("/", post(create_staff_role_handler))
}

async fn create_staff_role_handler(
    authorized_ctx: AuthorizedContext,
    Json(payload): Json<CreateStaffRole>,
) -> Result<ApiResponse<String>, AppError> {
    let ctx = authorized_ctx.require_permission(Permission::StaffRoleCreate)?;
    let role_public_id = StaffRoleService::create_staff_role(&ctx, payload).await?;
    Ok(ApiResponse::success(
        role_public_id,
        "Staff role created",
        Some(StatusCode::CREATED),
    ))
}
