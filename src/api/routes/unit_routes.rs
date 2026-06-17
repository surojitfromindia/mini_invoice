use super::openapi_docs;
use crate::api::AuthenticatedContext;
use crate::api::api_response::ApiResponse;
use crate::api::dto::common_dto::{IntoServiceInput, PublicIdResponse};
use crate::api::dto::unit_dto::{
    CreateUnitRequestDto, UnitListItemResponseDto, UnitListPageQueryDto,
};
use crate::app_state::AppState;
use crate::db::listing::PageListResult;
use crate::errors::app_error::AppError;
use crate::service::unit_service::UnitService;
use aide::axum::ApiRouter;
use axum::extract::Query as AxumQuery;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use axum_extra::extract::Query as AxumExtraQuery;

pub fn routes() -> ApiRouter<AppState> {
    ApiRouter::from(
        Router::new().route("/", post(create_unit_handler).get(list_units_page_handler)),
    )
    .api_route_docs(
        "/",
        openapi_docs::method("post", "unit", "createUnit", "Create unit", |op| {
            op.input::<Json<CreateUnitRequestDto>>();
            op.response::<201, ApiResponse<PublicIdResponse>>();
        }),
    )
    .api_route_docs(
        "/",
        openapi_docs::method("get", "unit", "listUnits", "List units", |op| {
            op.input::<AxumQuery<UnitListPageQueryDto>>();
            op.response::<200, ApiResponse<PageListResult<UnitListItemResponseDto>>>();
        }),
    )
}

async fn create_unit_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    Json(payload): Json<CreateUnitRequestDto>,
) -> Result<ApiResponse<PublicIdResponse>, AppError> {
    let public_id = UnitService::create_unit(&ctx, payload.into_service_input()).await?;

    Ok(ApiResponse::success(
        PublicIdResponse { public_id },
        "Unit created",
        Some(StatusCode::CREATED),
    ))
}

async fn list_units_page_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    AxumExtraQuery(query): AxumExtraQuery<UnitListPageQueryDto>,
) -> Result<ApiResponse<PageListResult<UnitListItemResponseDto>>, AppError> {
    let result = UnitService::list_units_page(&ctx, query.into_service_input()).await?;

    Ok(ApiResponse::success(
        UnitListItemResponseDto::page_from_service_output(result),
        "Units fetched",
        Some(StatusCode::OK),
    ))
}
