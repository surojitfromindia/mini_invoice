use std::future::Future;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;

use crate::app_state::AppState;
use crate::auth::authorization_service::AuthorizationService;
use crate::auth::permission::Permission;
use crate::errors::app_error::AppError;
use crate::service::service_context::ServiceContext;

use super::context::AuthenticatedContext;

// This extractor guarantees the request is authenticated.
// Handlers can then declare the permission rule they need without creating
// a new marker type for every single permission or permission combination.
pub struct AuthorizedContext(ServiceContext);

impl AuthorizedContext {
    pub fn into_context(self) -> ServiceContext {
        self.0
    }

    pub fn require_permission(self, permission: Permission) -> Result<ServiceContext, AppError> {
        AuthorizationService::require_permission(&self.0, permission)?;
        Ok(self.0)
    }

    pub fn require_all<const N: usize>(
        self,
        permissions: [Permission; N],
    ) -> Result<ServiceContext, AppError> {
        AuthorizationService::require_all_permissions(&self.0, &permissions)?;
        Ok(self.0)
    }

    pub fn require_any<const N: usize>(
        self,
        permissions: [Permission; N],
    ) -> Result<ServiceContext, AppError> {
        AuthorizationService::require_any_permission(&self.0, &permissions)?;
        Ok(self.0)
    }
}

impl FromRequestParts<AppState> for AuthorizedContext {
    type Rejection = AppError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let authenticated_context = AuthenticatedContext::from_request_parts(parts, state);

        async move {
            let AuthenticatedContext(ctx) = authenticated_context.await?;
            Ok(Self(ctx))
        }
    }
}
