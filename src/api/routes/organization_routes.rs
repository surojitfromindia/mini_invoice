use super::openapi_docs;
use crate::api::AuthenticatedContext;
use crate::api::api_response::ApiResponse;
use crate::api::dto::common_dto::{IntoServiceInput, PublicIdResponse};
use crate::api::dto::organization_dto::CreateOrganizationRequestDto;
use crate::app_state::AppState;
use crate::errors::app_error::AppError;
use crate::service::organization_service::OrganizationService;
use aide::axum::ApiRouter;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};

pub fn routes() -> ApiRouter<AppState> {
    ApiRouter::from(Router::new().route("/basic", post(create_organization_handler)))
        .api_route_docs(
            "/basic",
            openapi_docs::method(
                "post",
                "organization",
                "createOrganization",
                "Create organization",
                |op| {
                    op.input::<Json<CreateOrganizationRequestDto>>();
                    op.response::<201, ApiResponse<PublicIdResponse>>();
                },
            ),
        )
}

async fn create_organization_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    Json(payload): Json<CreateOrganizationRequestDto>,
) -> Result<ApiResponse<PublicIdResponse>, AppError> {
    let public_id =
        OrganizationService::create_organization(&ctx, payload.into_service_input()).await?;
    Ok(ApiResponse::success(
        PublicIdResponse { public_id },
        "Organization created",
        Some(StatusCode::CREATED),
    ))
}
