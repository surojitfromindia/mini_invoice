use super::openapi_docs;
use crate::api::AuthenticatedContext;
use crate::api::api_response::ApiResponse;
use crate::api::dto::coa_dto::{
    ChartOfAccountItemDto, ChartOfAccountsQueryDto, ChartOfAccountsResponseDto,
    CreateChartOfAccountRequestDto,
};
use crate::api::dto::common_dto::{ActionStatusResponse, IntoServiceInput, PublicIdResponse};
use crate::app_state::AppState;
use crate::errors::app_error::AppError;
use crate::resolver::coa_payload_resolver::CoaPayloadResolver;
use crate::service::coa_service::CoaService;
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
                post(create_chart_of_account_handler).get(get_chart_of_accounts_handler),
            )
            .route(
                "/{public_id}",
                axum::routing::get(get_chart_of_account_handler)
                    .delete(delete_chart_of_account_handler),
            ),
    )
    .api_route_docs(
        "/",
        openapi_docs::method(
            "post",
            "chartOfAccounts",
            "createChartOfAccount",
            "Create chart of account",
            |op| {
                op.input::<Json<CreateChartOfAccountRequestDto>>();
                op.response::<201, ApiResponse<PublicIdResponse>>();
            },
        ),
    )
    .api_route_docs(
        "/",
        openapi_docs::method(
            "get",
            "chartOfAccounts",
            "getChartOfAccounts",
            "Get chart of accounts",
            |op| {
                op.input::<AxumQuery<ChartOfAccountsQueryDto>>();
                op.response::<200, ApiResponse<ChartOfAccountsResponseDto>>();
            },
        ),
    )
    .api_route_docs(
        "/{public_id}",
        openapi_docs::method(
            "get",
            "chartOfAccounts",
            "getChartOfAccount",
            "Get chart of account",
            |op| {
                op.response::<200, ApiResponse<ChartOfAccountItemDto>>();
            },
        ),
    )
    .api_route_docs(
        "/{public_id}",
        openapi_docs::method(
            "delete",
            "chartOfAccounts",
            "deleteChartOfAccount",
            "Delete chart of account",
            |op| {
                op.response::<200, ApiResponse<ActionStatusResponse>>();
            },
        ),
    )
}

async fn create_chart_of_account_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    Json(payload): Json<CreateChartOfAccountRequestDto>,
) -> Result<ApiResponse<PublicIdResponse>, AppError> {
    let organization_id = ctx.get_organization_id()?;
    let resolved_payload = CoaPayloadResolver::create_account(
        &ctx.app_state.primary_write_replica,
        organization_id,
        payload.into_resolution_input(),
    )
    .await?;
    let public_id = CoaService::create_account(&ctx, resolved_payload).await?;

    Ok(ApiResponse::success(
        PublicIdResponse { public_id },
        "Chart of account created",
        Some(StatusCode::CREATED),
    ))
}

async fn get_chart_of_accounts_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    AxumExtraQuery(query): AxumExtraQuery<ChartOfAccountsQueryDto>,
) -> Result<ApiResponse<ChartOfAccountsResponseDto>, AppError> {
    let view_mode = query.view.unwrap_or_default().into_service_input();
    let chart = CoaService::fetch_default_chart_of_accounts(&ctx, view_mode).await?;

    Ok(ApiResponse::success(
        ChartOfAccountsResponseDto::from_service_output(chart),
        "Chart of accounts fetched",
        Some(StatusCode::OK),
    ))
}

async fn get_chart_of_account_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    Path(public_id): Path<String>,
) -> Result<ApiResponse<ChartOfAccountItemDto>, AppError> {
    let account = CoaService::get_account(&ctx, &public_id).await?;

    Ok(ApiResponse::success(
        ChartOfAccountItemDto::from_service_output(account),
        "Chart of account fetched",
        Some(StatusCode::OK),
    ))
}

async fn delete_chart_of_account_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    Path(public_id): Path<String>,
) -> Result<ApiResponse<ActionStatusResponse>, AppError> {
    CoaService::delete_account(&ctx, &public_id).await?;

    Ok(ApiResponse::success(
        ActionStatusResponse {
            status: "deleted".to_string(),
        },
        "Chart of account deleted",
        Some(StatusCode::OK),
    ))
}
