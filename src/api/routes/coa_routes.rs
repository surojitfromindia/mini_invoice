use super::openapi_docs;
use crate::api::AuthenticatedContext;
use crate::api::api_response::ApiResponse;
use crate::api::dto::coa_dto::{ChartOfAccountsQueryDto, ChartOfAccountsResponseDto};
use crate::api::dto::common_dto::IntoServiceInput;
use crate::app_state::AppState;
use crate::errors::app_error::AppError;
use crate::service::coa_service::CoaService;
use aide::axum::ApiRouter;
use axum::Router;
use axum::extract::Query as AxumQuery;
use axum::http::StatusCode;
use axum_extra::extract::Query as AxumExtraQuery;

pub fn routes() -> ApiRouter<AppState> {
    ApiRouter::from(Router::new().route("/", axum::routing::get(get_chart_of_accounts_handler)))
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
