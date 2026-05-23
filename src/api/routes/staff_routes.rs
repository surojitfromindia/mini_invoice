use crate::api::api_response::ApiResponse;
use crate::api::{AuthenticatedContext, PublicContext};
use crate::app_state::AppState;
use crate::errors::app_error::AppError;
use crate::resolver::staff_payload_resolver::StaffPayloadResolver;
use crate::service::staff_service::{
    AcceptStaffInvitation, CreateStaffInvitation, ResendStaffInvitation, RevokeStaffInvitation,
    StaffInvitationCreated, StaffService,
};
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde::Serialize;

#[derive(Serialize)]
struct StaffInvitationResponse {
    invitation_id: String,
    invitation_token: String,
    token_expires_at: chrono::DateTime<chrono::Utc>,
}

impl From<StaffInvitationCreated> for StaffInvitationResponse {
    fn from(value: StaffInvitationCreated) -> Self {
        Self {
            invitation_id: value.invitation_id,
            invitation_token: value.invitation_token,
            token_expires_at: value.token_expires_at,
        }
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/invitations", post(create_staff_invitation_handler))
        .route("/invitations/accept", post(accept_staff_invitation_handler))
        .route("/invitations/resend", post(resend_staff_invitation_handler))
        .route("/invitations/revoke", post(revoke_staff_invitation_handler))
}

async fn create_staff_invitation_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    Json(payload): Json<CreateStaffInvitation>,
) -> Result<ApiResponse<StaffInvitationResponse>, AppError> {
    let organization_id = ctx.get_organization_id()?;
    let resolved_payload = StaffPayloadResolver::resolve_create_staff_invitation(
        &ctx.app_state.primary_read_replica,
        organization_id,
        payload,
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
    Json(payload): Json<AcceptStaffInvitation>,
) -> Result<ApiResponse<String>, AppError> {
    StaffService::accept_staff_invitation(&ctx, payload).await?;
    Ok(ApiResponse::success(
        "Invitation accepted".to_string(),
        "Staff invitation accepted",
        Some(StatusCode::CREATED),
    ))
}

async fn resend_staff_invitation_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    Json(payload): Json<ResendStaffInvitation>,
) -> Result<ApiResponse<StaffInvitationResponse>, AppError> {
    let organization_id = ctx.get_organization_id()?;
    let invitation_model = StaffPayloadResolver::resolve_resend_staff_invitation(
        &ctx.app_state.primary_read_replica,
        organization_id,
        payload,
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
    AuthenticatedContext(ctx): AuthenticatedContext,
    Json(payload): Json<RevokeStaffInvitation>,
) -> Result<ApiResponse<String>, AppError> {
    let organization_id = ctx.get_organization_id()?;
    let invitation_model = StaffPayloadResolver::resolve_revoke_staff_invitation(
        &ctx.app_state.primary_read_replica,
        organization_id,
        payload,
    )
    .await?;
    StaffService::revoke_staff_invitation(&ctx, invitation_model).await?;
    Ok(ApiResponse::success(
        "Invitation revoked".to_string(),
        "Staff invitation revoked",
        Some(StatusCode::OK),
    ))
}
