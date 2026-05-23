use crate::api::api_response::ApiResponse;
use crate::api::dto::common_dto::ActionStatusResponse;
use crate::api::dto::staff_dto::{
    AcceptStaffInvitationRequestDto, CreateStaffInvitationRequestDto,
    ResendStaffInvitationRequestDto, RevokeStaffInvitationRequestDto, StaffInvitationResponseDto,
    accepted_response, revoked_response,
};
use crate::api::{AuthorizedContext, PublicContext};
use crate::app_state::AppState;
use crate::auth::permission::Permission;
use crate::errors::app_error::AppError;
use crate::resolver::staff_payload_resolver::StaffPayloadResolver;
use crate::service::staff_service::StaffService;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/invitations", post(create_staff_invitation_handler))
        .route("/invitations/accept", post(accept_staff_invitation_handler))
        .route("/invitations/resend", post(resend_staff_invitation_handler))
        .route("/invitations/revoke", post(revoke_staff_invitation_handler))
}

async fn create_staff_invitation_handler(
    authorized_ctx: AuthorizedContext,
    Json(payload): Json<CreateStaffInvitationRequestDto>,
) -> Result<ApiResponse<StaffInvitationResponseDto>, AppError> {
    let ctx = authorized_ctx.require_any([Permission::StaffInvite])?;
    let organization_id = ctx.get_organization_id()?;
    let resolved_payload = StaffPayloadResolver::resolve_create_staff_invitation(
        &ctx.app_state.primary_write_replica,
        organization_id,
        payload.into(),
    )
    .await?;
    let invitation = StaffService::create_staff_invitation(&ctx, resolved_payload).await?;
    Ok(ApiResponse::success(
        invitation.into(),
        "Staff invitation created",
        Some(StatusCode::CREATED),
    ))
}

async fn accept_staff_invitation_handler(
    PublicContext(ctx): PublicContext,
    Json(payload): Json<AcceptStaffInvitationRequestDto>,
) -> Result<ApiResponse<ActionStatusResponse>, AppError> {
    StaffService::accept_staff_invitation(&ctx, payload.into()).await?;
    Ok(ApiResponse::success(
        accepted_response(),
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
    let invitation_model = StaffPayloadResolver::resolve_resend_staff_invitation(
        &ctx.app_state.primary_write_replica,
        organization_id,
        payload.into(),
    )
    .await?;
    let invitation = StaffService::resend_staff_invitation(&ctx, invitation_model).await?;
    Ok(ApiResponse::success(
        invitation.into(),
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
    let invitation_model = StaffPayloadResolver::resolve_revoke_staff_invitation(
        &ctx.app_state.primary_write_replica,
        organization_id,
        payload.into(),
    )
    .await?;
    StaffService::revoke_staff_invitation(&ctx, invitation_model).await?;
    Ok(ApiResponse::success(
        revoked_response(),
        "Staff invitation revoked",
        Some(StatusCode::OK),
    ))
}
