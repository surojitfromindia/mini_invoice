use crate::api::AuthorizedContext;
use crate::api::api_response::ApiResponse;
use crate::api::dto::branch_dto::{
    BranchListItemResponseDto, BranchListPageQueryDto, CreateBranchRequestDto,
};
use crate::api::dto::common_dto::PublicIdResponse;
use crate::app_state::AppState;
use crate::auth::permission::Permission;
use crate::db::listing::PageListResult;
use crate::errors::app_error::AppError;
use crate::service::branch_service::BranchService;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};

pub fn routes() -> Router<AppState> {
    Router::new().route(
        "/",
        post(create_branch_handler).get(list_branches_page_handler),
    )
}

async fn create_branch_handler(
    authorized_ctx: AuthorizedContext,
    Json(payload): Json<CreateBranchRequestDto>,
) -> Result<ApiResponse<PublicIdResponse>, AppError> {
    let ctx = authorized_ctx.require_all([Permission::BranchCreate])?;
    let public_id = BranchService::create_branch(&ctx, payload.into_service_input()).await?;
    Ok(ApiResponse::success(
        PublicIdResponse { public_id },
        "Branch created",
        Some(StatusCode::CREATED),
    ))
}

async fn list_branches_page_handler(
    authorized_ctx: AuthorizedContext,
    Query(query): Query<BranchListPageQueryDto>,
) -> Result<ApiResponse<PageListResult<BranchListItemResponseDto>>, AppError> {
    let ctx = authorized_ctx.into_context();
    let result = BranchService::list_branches_page(&ctx, query.into_service_input()).await?;
    Ok(ApiResponse::success(
        BranchListItemResponseDto::page_from_service_output(result),
        "Branches fetched",
        Some(StatusCode::OK),
    ))
}
