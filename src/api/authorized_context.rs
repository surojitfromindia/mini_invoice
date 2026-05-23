use std::future::Future;
use std::marker::PhantomData;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::app_state::AppState;
use crate::auth::authorization_service::AuthorizationService;
use crate::auth::permission::Permission;
use crate::errors::app_error::AppError;
use crate::service::service_context::ServiceContext;

use super::context::AuthenticatedContext;

pub trait PermissionRequirement {
    const PERMISSION: Permission;
}

pub struct AuthorizedContext<P>(pub ServiceContext, PhantomData<P>);

impl<P> AuthorizedContext<P> {
    pub fn into_service_context(self) -> ServiceContext {
        self.0
    }
}

impl<P> FromRequestParts<AppState> for AuthorizedContext<P>
where
    P: PermissionRequirement + Send + Sync,
{
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let authenticated_context = AuthenticatedContext::from_request_parts(parts, state);

        async move {
            let AuthenticatedContext(ctx) = authenticated_context.await?;
            // Handlers opt into a permission marker, and the extractor blocks the
            // request before any business service code runs when access is missing.
            AuthorizationService::require_permission(&ctx, P::PERMISSION).await?;
            Ok(Self(ctx, PhantomData))
        }
    }
}

pub struct BranchCreatePermission;
impl PermissionRequirement for BranchCreatePermission {
    const PERMISSION: Permission = Permission::BranchCreate;
}

pub struct StaffInvitePermission;
impl PermissionRequirement for StaffInvitePermission {
    const PERMISSION: Permission = Permission::StaffInvite;
}

pub struct StaffInvitationResendPermission;
impl PermissionRequirement for StaffInvitationResendPermission {
    const PERMISSION: Permission = Permission::StaffInvitationResend;
}

pub struct StaffInvitationRevokePermission;
impl PermissionRequirement for StaffInvitationRevokePermission {
    const PERMISSION: Permission = Permission::StaffInvitationRevoke;
}

pub struct StaffRoleCreatePermission;
impl PermissionRequirement for StaffRoleCreatePermission {
    const PERMISSION: Permission = Permission::StaffRoleCreate;
}
