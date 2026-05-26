use crate::api::AuthenticatedContext;
use crate::api::api_response::ApiResponse;
use crate::api::dto::common_dto::{IntoServiceInput, PublicIdResponse};
use crate::api::dto::item_dto::{
    CreateItemRequestDto, ItemListItemResponseDto, ItemListPageQueryDto,
};
use crate::app_state::AppState;
use crate::db::listing::PageListResult;
use crate::errors::app_error::AppError;
use crate::resolver::item_payload_resolver::ItemPayloadResolver;
use crate::service::item_service::ItemService;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use axum_extra::extract::Query;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", post(create_item_handler).get(list_items_page_handler))
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

async fn list_items_page_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    Query(query): Query<ItemListPageQueryDto>,
) -> Result<ApiResponse<PageListResult<ItemListItemResponseDto>>, AppError> {
    let result = ItemService::list_items_page(&ctx, query.into_service_input()).await?;

    Ok(ApiResponse::success(
        ItemListItemResponseDto::page_from_service_output(result),
        "Items fetched",
        Some(StatusCode::OK),
    ))
}
