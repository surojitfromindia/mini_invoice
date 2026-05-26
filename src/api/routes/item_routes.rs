use crate::api::api_response::ApiResponse;
use crate::api::dto::common_dto::PublicIdResponse;
use crate::api::dto::item_dto::CreateItemRequestDto;
use crate::api::AuthenticatedContext;
use crate::app_state::AppState;
use crate::errors::app_error::AppError;
use crate::resolver::item_payload_resolver::ItemPayloadResolver;
use crate::service::item_service::ItemService;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};

pub fn routes() -> Router<AppState> {
    Router::new().route("/", post(create_item_handler))
}

async fn create_item_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    Json(payload): Json<CreateItemRequestDto>,
) -> Result<ApiResponse<PublicIdResponse>, AppError> {
    let organization_id = ctx.get_organization_id()?;
    let resolved_payload = ItemPayloadResolver::create_item(
        &ctx.app_state.primary_write_replica,
        organization_id,
        payload.into_resolution_input(),
    )
    .await?;
    let public_id = ItemService::create_item(&ctx, resolved_payload).await?;

    Ok(ApiResponse::success(
        PublicIdResponse { public_id },
        "Item created",
        Some(StatusCode::CREATED),
    ))
}
