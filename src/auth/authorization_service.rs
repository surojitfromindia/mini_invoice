use crate::auth::permission::{
    Permission, has_all_permissions, has_any_permission, missing_permissions,
};
use crate::errors::app_error::AppError;
use crate::service::service_context::ServiceContext;

pub struct AuthorizationService;

impl AuthorizationService {
    pub fn require_permission(
        ctx: &ServiceContext,
        permission: Permission,
    ) -> Result<(), AppError> {
        Self::require_all_permissions(ctx, &[permission])
    }

    pub fn require_all_permissions(
        ctx: &ServiceContext,
        permissions: &[Permission],
    ) -> Result<(), AppError> {
        let Some(staff_access) = ctx.get_staff_access() else {
            return Err(AppError::Unauthorized);
        };

        if permissions.is_empty()
            || has_all_permissions(&staff_access.permission_code_set, permissions)
        {
            return Ok(());
        }

        let missing_permissions =
            missing_permissions(&staff_access.permission_code_set, permissions);

        Err(AppError::Forbidden {
            permission: format_permission_requirement("all of", &missing_permissions),
        })
    }

    pub fn require_any_permission(
        ctx: &ServiceContext,
        permissions: &[Permission],
    ) -> Result<(), AppError> {
        let Some(staff_access) = ctx.get_staff_access() else {
            return Err(AppError::Unauthorized);
        };

        if permissions.is_empty()
            || has_any_permission(&staff_access.permission_code_set, permissions)
        {
            return Ok(());
        }

        Err(AppError::Forbidden {
            permission: format_permission_requirement("one of", permissions),
        })
    }
}

fn format_permission_requirement(prefix: &str, permissions: &[Permission]) -> String {
    if permissions.len() == 1 {
        return permissions[0].code().to_string();
    }

    let codes = permissions
        .iter()
        .map(Permission::code)
        .collect::<Vec<_>>()
        .join(", ");

    format!("{prefix} [{codes}]")
}
