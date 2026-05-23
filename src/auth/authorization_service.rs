use crate::auth::permission::{Permission, has_permission};
use crate::errors::app_error::AppError;
use crate::service::service_context::ServiceContext;

pub struct AuthorizationService;

impl AuthorizationService {
    pub async fn require_permission(
        ctx: &ServiceContext,
        permission: Permission,
    ) -> Result<(), AppError> {
        let Some(staff_access) = ctx.get_staff_access() else {
            return Err(AppError::Unauthorized);
        };

        if has_permission(&staff_access.permission_codes, permission) {
            return Ok(());
        }

        Err(AppError::Forbidden {
            permission: permission.code(),
        })
    }
}
