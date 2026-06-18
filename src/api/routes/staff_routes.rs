use super::openapi_docs;
use crate::api::api_response::ApiResponse;
use crate::api::dto::common_dto::{ActionStatusResponse, IntoServiceInput};
use crate::api::dto::staff_dto::{
    AcceptStaffInvitationRequestDto, CreateStaffInvitationRequestDto,
    ResendStaffInvitationRequestDto, RevokeStaffInvitationRequestDto, StaffInvitationResponseDto,
    StaffListItemResponseDto, StaffListPageQueryDto, StaffResponseDto, UpdateStaffRequestDto,
};
use crate::api::{AuthorizedContext, PublicContext};
use crate::app_state::AppState;
use crate::auth::permission::Permission;
use crate::db::listing::PageListResult;
use crate::errors::app_error::AppError;
use crate::resolver::public_id_resolver::PublicIdResolver;
use crate::resolver::staff_payload_resolver::{
    CreateStaffInvitationResolutionInput, StaffPayloadResolver,
};
use crate::service::staff_service::StaffService;
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
            .route("/", axum::routing::get(list_staff_page_handler))
            .route(
                "/{public_id}",
                axum::routing::get(get_staff_handler)
                    .put(update_staff_handler)
                    .delete(delete_staff_handler),
            )
            .route("/invitations", post(create_staff_invitation_handler))
            .route("/invitations/accept", post(accept_staff_invitation_handler))
            .route("/invitations/resend", post(resend_staff_invitation_handler))
            .route("/invitations/revoke", post(revoke_staff_invitation_handler)),
    )
    .api_route_docs(
        "/",
        openapi_docs::method("get", "staff", "listStaff", "List staff", |op| {
            op.input::<AxumQuery<StaffListPageQueryDto>>();
            op.response::<200, ApiResponse<PageListResult<StaffListItemResponseDto>>>();
        }),
    )
    .api_route_docs(
        "/{public_id}",
        openapi_docs::method("get", "staff", "getStaff", "Get staff", |op| {
            op.response::<200, ApiResponse<StaffResponseDto>>();
        }),
    )
    .api_route_docs(
        "/{public_id}",
        openapi_docs::method("put", "staff", "updateStaff", "Update staff", |op| {
            op.input::<Json<UpdateStaffRequestDto>>();
            op.response::<200, ApiResponse<StaffResponseDto>>();
        }),
    )
    .api_route_docs(
        "/{public_id}",
        openapi_docs::method("delete", "staff", "deleteStaff", "Delete staff", |op| {
            op.response::<200, ApiResponse<ActionStatusResponse>>();
        }),
    )
    .api_route_docs(
        "/invitations",
        openapi_docs::method(
            "post",
            "staff",
            "createStaffInvitation",
            "Create staff invitation",
            |op| {
                op.input::<Json<CreateStaffInvitationRequestDto>>();
                op.response::<201, ApiResponse<StaffInvitationResponseDto>>();
            },
        ),
    )
    .api_route_docs(
        "/invitations/accept",
        openapi_docs::method(
            "post",
            "staff",
            "acceptStaffInvitation",
            "Accept staff invitation",
            |op| {
                op.input::<Json<AcceptStaffInvitationRequestDto>>();
                op.response::<201, ApiResponse<ActionStatusResponse>>();
            },
        ),
    )
    .api_route_docs(
        "/invitations/resend",
        openapi_docs::method(
            "post",
            "staff",
            "resendStaffInvitation",
            "Resend staff invitation",
            |op| {
                op.input::<Json<ResendStaffInvitationRequestDto>>();
                op.response::<200, ApiResponse<StaffInvitationResponseDto>>();
            },
        ),
    )
    .api_route_docs(
        "/invitations/revoke",
        openapi_docs::method(
            "post",
            "staff",
            "revokeStaffInvitation",
            "Revoke staff invitation",
            |op| {
                op.input::<Json<RevokeStaffInvitationRequestDto>>();
                op.response::<200, ApiResponse<ActionStatusResponse>>();
            },
        ),
    )
}

async fn list_staff_page_handler(
    authorized_ctx: AuthorizedContext,
    AxumExtraQuery(query): AxumExtraQuery<StaffListPageQueryDto>,
) -> Result<ApiResponse<PageListResult<StaffListItemResponseDto>>, AppError> {
    let ctx = authorized_ctx.require_permission(Permission::StaffRead)?;
    let result = StaffService::list_staff_page(&ctx, query.into_service_input()).await?;
    Ok(ApiResponse::success(
        StaffListItemResponseDto::page_from_service_output(result),
        "Staff fetched",
        Some(StatusCode::OK),
    ))
}

async fn get_staff_handler(
    authorized_ctx: AuthorizedContext,
    Path(public_id): Path<String>,
) -> Result<ApiResponse<StaffResponseDto>, AppError> {
    let ctx = authorized_ctx.require_permission(Permission::StaffRead)?;
    let result = StaffService::get_staff(&ctx, &public_id).await?;
    Ok(ApiResponse::success(
        StaffResponseDto::from_service_output(result),
        "Staff fetched",
        Some(StatusCode::OK),
    ))
}

async fn update_staff_handler(
    authorized_ctx: AuthorizedContext,
    Path(public_id): Path<String>,
    Json(payload): Json<UpdateStaffRequestDto>,
) -> Result<ApiResponse<StaffResponseDto>, AppError> {
    let ctx = authorized_ctx.require_permission(Permission::StaffUpdate)?;
    let organization_id = ctx.get_organization_id()?;
    let resolved_payload = StaffPayloadResolver::update_staff(
        &ctx.app_state.primary_write_replica,
        organization_id,
        payload.into_resolution_input(),
    )
    .await?;
    let result = StaffService::update_staff(&ctx, &public_id, resolved_payload).await?;
    Ok(ApiResponse::success(
        StaffResponseDto::from_service_output(result),
        "Staff updated",
        Some(StatusCode::OK),
    ))
}

async fn delete_staff_handler(
    authorized_ctx: AuthorizedContext,
    Path(public_id): Path<String>,
) -> Result<ApiResponse<ActionStatusResponse>, AppError> {
    let ctx = authorized_ctx.require_permission(Permission::StaffDelete)?;
    StaffService::delete_staff(&ctx, &public_id).await?;
    Ok(ApiResponse::success(
        ActionStatusResponse {
            status: "deleted".to_string(),
        },
        "Staff deleted",
        Some(StatusCode::OK),
    ))
}

async fn create_staff_invitation_handler(
    authorized_ctx: AuthorizedContext,
    Json(payload): Json<CreateStaffInvitationRequestDto>,
) -> Result<ApiResponse<StaffInvitationResponseDto>, AppError> {
    let ctx = authorized_ctx.require_any([Permission::StaffInvite])?;
    let organization_id = ctx.get_organization_id()?;
    let resolved_payload = StaffPayloadResolver::create_staff_invitation(
        &ctx.app_state.primary_write_replica,
        organization_id,
        CreateStaffInvitationResolutionInput {
            invitee_email: payload.invitee_email,
            invitee_first_name: payload.invitee_first_name,
            invitee_last_name: payload.invitee_last_name,
            role_public_id: payload.role_public_id,
            branch_public_ids: payload.branch_public_ids,
        },
    )
    .await?;
    let invitation = StaffService::create_staff_invitation(&ctx, resolved_payload).await?;
    Ok(ApiResponse::success(
        StaffInvitationResponseDto::from_service_output(invitation),
        "Staff invitation created",
        Some(StatusCode::CREATED),
    ))
}

async fn accept_staff_invitation_handler(
    PublicContext(ctx): PublicContext,
    Json(payload): Json<AcceptStaffInvitationRequestDto>,
) -> Result<ApiResponse<ActionStatusResponse>, AppError> {
    StaffService::accept_staff_invitation(&ctx, payload.into_service_input()).await?;
    Ok(ApiResponse::success(
        ActionStatusResponse {
            status: "invitation_accepted".to_string(),
        },
        "Staff invitation accepted",
        Some(StatusCode::CREATED),
    ))
}

async fn resend_staff_invitation_handler(
    authorized_ctx: AuthorizedContext,
    Json(payload): Json<ResendStaffInvitationRequestDto>,
) -> Result<ApiResponse<StaffInvitationResponseDto>, AppError> {
    let ctx = authorized_ctx.require_permission(Permission::StaffInvitationResend)?;
    let organization_id = ctx.get_organization_id()?;
    let invitation_id = PublicIdResolver::staff_invitation_id(
        &ctx.app_state.primary_write_replica,
        organization_id,
        &payload.invitation_id,
    )
    .await?;
    let invitation = StaffService::resend_staff_invitation(&ctx, invitation_id).await?;
    Ok(ApiResponse::success(
        StaffInvitationResponseDto::from_service_output(invitation),
        "Staff invitation resent",
        Some(StatusCode::OK),
    ))
}

async fn revoke_staff_invitation_handler(
    authorized_ctx: AuthorizedContext,
    Json(payload): Json<RevokeStaffInvitationRequestDto>,
) -> Result<ApiResponse<ActionStatusResponse>, AppError> {
    let ctx = authorized_ctx.require_permission(Permission::StaffInvitationRevoke)?;
    let organization_id = ctx.get_organization_id()?;
    let invitation_id = PublicIdResolver::staff_invitation_id(
        &ctx.app_state.primary_write_replica,
        organization_id,
        &payload.invitation_id,
    )
    .await?;
    StaffService::revoke_staff_invitation(&ctx, invitation_id).await?;
    Ok(ApiResponse::success(
        ActionStatusResponse {
            status: "invitation_revoked".to_string(),
        },
        "Staff invitation revoked",
        Some(StatusCode::OK),
    ))
}
