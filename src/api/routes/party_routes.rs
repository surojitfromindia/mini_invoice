use super::openapi_docs;
use crate::api::AuthenticatedContext;
use crate::api::api_response::ApiResponse;
use crate::api::dto::common_dto::{IntoServiceInput, PublicIdResponse};
use crate::api::dto::party_dto::{
    CreatePartyRequestDto, PartyListItemResponseDto, PartyListPageQueryDto,
};
use crate::app_state::AppState;
use crate::db::listing::PageListResult;
use crate::errors::app_error::AppError;
use crate::resolver::party_payload_resolver::PartyPayloadResolver;
use crate::service::party_service::PartyService;
use aide::axum::ApiRouter;
use axum::extract::Query as AxumQuery;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use axum_extra::extract::Query as AxumExtraQuery;

pub fn routes() -> ApiRouter<AppState> {
    ApiRouter::from(Router::new().route(
        "/",
        post(create_party_handler).get(list_parties_page_handler),
    ))
    .api_route_docs(
        "/",
        openapi_docs::method("post", "party", "createParty", "Create party", |op| {
            op.input::<Json<CreatePartyRequestDto>>();
            op.response::<201, ApiResponse<PublicIdResponse>>();
        }),
    )
    .api_route_docs(
        "/",
        openapi_docs::method("get", "party", "listParties", "List parties", |op| {
            op.input::<AxumQuery<PartyListPageQueryDto>>();
            op.response::<200, ApiResponse<PageListResult<PartyListItemResponseDto>>>();
        }),
    )
}

async fn create_party_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    Json(payload): Json<CreatePartyRequestDto>,
) -> Result<ApiResponse<PublicIdResponse>, AppError> {
    let organization_id = ctx.get_organization_id()?;
    let resolved_payload = PartyPayloadResolver::create_party(
        &ctx.app_state.primary_write_replica,
        organization_id,
        payload.into_resolution_input(),
    )
    .await?;
    let public_id = PartyService::create_party(&ctx, resolved_payload).await?;

    Ok(ApiResponse::success(
        PublicIdResponse { public_id },
        "Party created",
        Some(StatusCode::CREATED),
    ))
}

async fn list_parties_page_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    AxumExtraQuery(query): AxumExtraQuery<PartyListPageQueryDto>,
) -> Result<ApiResponse<PageListResult<PartyListItemResponseDto>>, AppError> {
    let result = PartyService::list_parties_page(&ctx, query.into_service_input()).await?;

    Ok(ApiResponse::success(
        PartyListItemResponseDto::page_from_service_output(result),
        "Parties fetched",
        Some(StatusCode::OK),
    ))
}
