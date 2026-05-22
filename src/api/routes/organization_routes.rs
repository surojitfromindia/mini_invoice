use crate::api::AuthenticatedContext;
use crate::api::api_response::ApiResponse;
use crate::app_state::AppState;
use crate::errors::app_error::AppError;
use crate::service::organization_service::{CreateOrganization, OrganizationService};
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};

pub fn routes() -> Router<AppState> {
    Router::new().route("/basic", post(create_organization_handler))
}

async fn create_organization_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    Json(payload): Json<CreateOrganization>,
) -> Result<ApiResponse<String>, AppError> {
    let public_id = OrganizationService::create_organization(&ctx, payload).await?;
    Ok(ApiResponse::success(
        public_id,
        "Organization created",
        Some(StatusCode::CREATED),
    ))
}
