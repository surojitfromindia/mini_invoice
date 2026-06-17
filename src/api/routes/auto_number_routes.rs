use super::openapi_docs;
use crate::api::AuthenticatedContext;
use crate::api::api_response::ApiResponse;
use crate::api::dto::auto_number_dto::{
    AutoNumberSeriesListPageQueryDto, AutoNumberSeriesResponseDto,
    CreateAutoNumberSeriesRequestDto, UpdateAutoNumberSeriesRequestDto,
};
use crate::api::dto::common_dto::{ActionStatusResponse, PublicIdResponse};
use crate::app_state::AppState;
use crate::db::listing::PageListResult;
use crate::entity::PrimaryId;
use crate::errors::app_error::AppError;
use crate::resolver::public_id_resolver::PublicIdResolver;
use crate::service::auto_number_service::AutoNumberService;
use crate::service::service_context::ServiceContext;
use aide::axum::ApiRouter;
use axum::extract::Path;
use axum::extract::Query as AxumQuery;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use axum_extra::extract::Query as AxumExtraQuery;

pub fn routes() -> ApiRouter<AppState> {
    ApiRouter::from(
        Router::new()
            .route(
                "/",
                post(create_auto_number_series_handler).get(list_auto_number_series_handler),
            )
            .route(
                "/{public_id}",
                axum::routing::get(get_auto_number_series_handler)
                    .put(update_auto_number_series_handler)
                    .delete(delete_auto_number_series_handler),
            ),
    )
    .api_route_docs(
        "/",
        openapi_docs::method(
            "post",
            "autoNumber",
            "createAutoNumberSeries",
            "Create auto number series",
            |op| {
                op.input::<Json<CreateAutoNumberSeriesRequestDto>>();
                op.response::<201, ApiResponse<PublicIdResponse>>();
            },
        ),
    )
    .api_route_docs(
        "/",
        openapi_docs::method(
            "get",
            "autoNumber",
            "listAutoNumberSeries",
            "List auto number series",
            |op| {
                op.input::<AxumQuery<AutoNumberSeriesListPageQueryDto>>();
                op.response::<200, ApiResponse<PageListResult<AutoNumberSeriesResponseDto>>>();
            },
        ),
    )
    .api_route_docs(
        "/{public_id}",
        openapi_docs::method(
            "get",
            "autoNumber",
            "getAutoNumberSeries",
            "Get auto number series",
            |op| {
                op.response::<200, ApiResponse<AutoNumberSeriesResponseDto>>();
            },
        ),
    )
    .api_route_docs(
        "/{public_id}",
        openapi_docs::method(
            "put",
            "autoNumber",
            "updateAutoNumberSeries",
            "Update auto number series",
            |op| {
                op.input::<Json<UpdateAutoNumberSeriesRequestDto>>();
                op.response::<200, ApiResponse<AutoNumberSeriesResponseDto>>();
            },
        ),
    )
    .api_route_docs(
        "/{public_id}",
        openapi_docs::method(
            "delete",
            "autoNumber",
            "deleteAutoNumberSeries",
            "Delete auto number series",
            |op| {
                op.response::<200, ApiResponse<ActionStatusResponse>>();
            },
        ),
    )
}

async fn create_auto_number_series_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    Json(payload): Json<CreateAutoNumberSeriesRequestDto>,
) -> Result<ApiResponse<PublicIdResponse>, AppError> {
    let organization_id = ctx.get_organization_id()?;
    let payload = payload.into_resolution_input();
    let branch_id =
        resolve_branch_id(&ctx, organization_id, payload.branch_public_id.clone()).await?;
    let public_id =
        AutoNumberService::create_series(&ctx, payload.into_service_input(branch_id)).await?;

    Ok(ApiResponse::success(
        PublicIdResponse { public_id },
        "Auto number series created",
        Some(StatusCode::CREATED),
    ))
}

async fn list_auto_number_series_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    AxumExtraQuery(query): AxumExtraQuery<AutoNumberSeriesListPageQueryDto>,
) -> Result<ApiResponse<PageListResult<AutoNumberSeriesResponseDto>>, AppError> {
    let organization_id = ctx.get_organization_id()?;
    let query = query.into_resolution_input();
    let branch_id =
        resolve_optional_branch_id(&ctx, organization_id, query.branch_public_id.clone()).await?;
    let result =
        AutoNumberService::list_series_page(&ctx, query.into_service_input(branch_id)).await?;

    Ok(ApiResponse::success(
        AutoNumberSeriesResponseDto::page_from_service_output(result),
        "Auto number series fetched",
        Some(StatusCode::OK),
    ))
}

async fn get_auto_number_series_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    Path(public_id): Path<String>,
) -> Result<ApiResponse<AutoNumberSeriesResponseDto>, AppError> {
    let result = AutoNumberService::get_series(&ctx, &public_id).await?;

    Ok(ApiResponse::success(
        AutoNumberSeriesResponseDto::from_detail(result),
        "Auto number series fetched",
        Some(StatusCode::OK),
    ))
}

async fn update_auto_number_series_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    Path(public_id): Path<String>,
    Json(payload): Json<UpdateAutoNumberSeriesRequestDto>,
) -> Result<ApiResponse<AutoNumberSeriesResponseDto>, AppError> {
    let organization_id = ctx.get_organization_id()?;
    let payload = payload.into_resolution_input();
    let branch_id =
        resolve_optional_branch_id(&ctx, organization_id, payload.branch_public_id.clone()).await?;
    let result =
        AutoNumberService::update_series(&ctx, &public_id, payload.into_service_input(branch_id))
            .await?;

    Ok(ApiResponse::success(
        AutoNumberSeriesResponseDto::from_detail(result),
        "Auto number series updated",
        Some(StatusCode::OK),
    ))
}

async fn delete_auto_number_series_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    Path(public_id): Path<String>,
) -> Result<ApiResponse<ActionStatusResponse>, AppError> {
    AutoNumberService::delete_series(&ctx, &public_id).await?;

    Ok(ApiResponse::success(
        ActionStatusResponse {
            status: "deleted".to_string(),
        },
        "Auto number series deleted",
        Some(StatusCode::OK),
    ))
}

async fn resolve_branch_id(
    ctx: &ServiceContext,
    organization_id: PrimaryId,
    branch_public_id: String,
) -> Result<PrimaryId, AppError> {
    resolve_optional_branch_id(ctx, organization_id, Some(branch_public_id))
        .await?
        .ok_or_else(|| AppError::InternalServer("Failed to resolve auto number branch".into()))
}

async fn resolve_optional_branch_id(
    ctx: &ServiceContext,
    organization_id: PrimaryId,
    branch_public_id: Option<String>,
) -> Result<Option<PrimaryId>, AppError> {
    let Some(branch_public_id) = branch_public_id else {
        return Ok(None);
    };

    let branch_ids = PublicIdResolver::branch_ids(
        &ctx.app_state.primary_read_replica,
        organization_id,
        Some(&[branch_public_id]),
    )
    .await?;

    Ok(branch_ids.into_iter().next())
}
