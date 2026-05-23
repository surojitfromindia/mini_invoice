use crate::api::api_response::ApiResponse;
use crate::api::{AuthenticatedContext, PublicContext};
use crate::app_state::AppState;
use crate::errors::app_error::AppError;
use crate::service::organization_service::{CreateOrganization, OrganizationService};
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
        .route("/basic", post(create_organization_handler))
        .route("/staff/invitations", post(create_staff_invitation_handler))
        .route(
            "/staff/invitations/accept",
            post(accept_staff_invitation_handler),
        )
        .route(
            "/staff/invitations/resend",
            post(resend_staff_invitation_handler),
        )
        .route(
            "/staff/invitations/revoke",
            post(revoke_staff_invitation_handler),
        )
}

async fn create_organization_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    Json(payload): Json<CreateOrganization>,
) -> Result<ApiResponse<String>, AppError> {
    let public_id = OrganizationService::create_organization(&ctx, payload).await?;
    Ok(ApiResponse::success(
        public_id,
        "Organization created",
        Some(StatusCode::CREATED),
    ))
}

async fn create_staff_invitation_handler(
    AuthenticatedContext(ctx): AuthenticatedContext,
    Json(payload): Json<CreateStaffInvitation>,
) -> Result<ApiResponse<StaffInvitationResponse>, AppError> {
    let invitation = StaffService::create_staff_invitation(&ctx, payload).await?;
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
    let invitation = StaffService::resend_staff_invitation(&ctx, payload).await?;
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
    StaffService::revoke_staff_invitation(&ctx, payload).await?;
    Ok(ApiResponse::success(
        "Invitation revoked".to_string(),
        "Staff invitation revoked",
        Some(StatusCode::OK),
    ))
}
