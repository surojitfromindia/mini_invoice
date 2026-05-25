use crate::api::api_response::ApiResponse;
use crate::api::dto::common_dto::{ActionStatusResponse, IntoServiceInput};
use crate::api::dto::staff_dto::{
    AcceptStaffInvitationRequestDto, CreateStaffInvitationRequestDto,
    ResendStaffInvitationRequestDto, RevokeStaffInvitationRequestDto, StaffInvitationResponseDto,
};
use crate::api::{AuthorizedContext, PublicContext};
use crate::app_state::AppState;
use crate::auth::permission::Permission;
use crate::errors::app_error::AppError;
use crate::resolver::public_id_resolver::PublicIdResolver;
use crate::resolver::staff_payload_resolver::{
    CreateStaffInvitationResolutionInput, StaffPayloadResolver,
};
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
