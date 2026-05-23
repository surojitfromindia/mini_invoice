use std::collections::HashSet;

use crate::errors::app_error::AppError;
use crate::errors::staff_service_errors::StaffServiceError;

// Central permission catalog for organization staff RBAC.
// Keep all valid permission codes mapped here so validation, storage,
// and request-time authorization all share the same source of truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    BranchCreate,
    StaffInvite,
    StaffInvitationResend,
    StaffInvitationRevoke,
    StaffRoleCreate,
}

impl Permission {
    pub fn code(&self) -> &'static str {
        match self {
            Self::BranchCreate => "branch.create",
            Self::StaffInvite => "staff.invite",
            Self::StaffInvitationResend => "staff.invitation.resend",
            Self::StaffInvitationRevoke => "staff.invitation.revoke",
            Self::StaffRoleCreate => "staff.role.create",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "branch.create" => Some(Self::BranchCreate),
            "staff.invite" => Some(Self::StaffInvite),
            "staff.invitation.resend" => Some(Self::StaffInvitationResend),
            "staff.invitation.revoke" => Some(Self::StaffInvitationRevoke),
            "staff.role.create" => Some(Self::StaffRoleCreate),
            _ => None,
        }
    }

    pub fn all_codes() -> Vec<String> {
        [
            Self::BranchCreate,
            Self::StaffInvite,
            Self::StaffInvitationResend,
            Self::StaffInvitationRevoke,
            Self::StaffRoleCreate,
        ]
        .into_iter()
        .map(|permission| permission.code().to_string())
        .collect()
    }
}

// Build a set once so repeated checks stay cheap even when a role carries
// a large number of permission codes.
pub fn build_permission_code_set(permission_codes: &[String]) -> HashSet<String> {
    permission_codes.iter().cloned().collect()
}

// Fast single-permission lookup against the cached permission set.
pub fn has_permission(permission_codes: &HashSet<String>, permission: Permission) -> bool {
    permission_codes.contains(permission.code())
}

// Use this when a route or service requires every permission in the list.
pub fn has_all_permissions(permission_codes: &HashSet<String>, permissions: &[Permission]) -> bool {
    permissions
        .iter()
        .all(|permission| has_permission(permission_codes, *permission))
}

// Use this when any one of the listed permissions should grant access.
pub fn has_any_permission(permission_codes: &HashSet<String>, permissions: &[Permission]) -> bool {
    permissions
        .iter()
        .any(|permission| has_permission(permission_codes, *permission))
}

// Return only the missing permissions so forbidden errors can explain
// exactly which requirements were not satisfied.
pub fn missing_permissions(
    permission_codes: &HashSet<String>,
    permissions: &[Permission],
) -> Vec<Permission> {
    permissions
        .iter()
        .copied()
        .filter(|permission| !has_permission(permission_codes, *permission))
        .collect()
}

// Normalize user-provided permission codes before storage.
// This trims whitespace, rejects unknown codes, and removes duplicates
// while preserving the first valid occurrence order.
pub fn normalize_permission_codes(permission_codes: &[String]) -> Result<Vec<String>, AppError> {
    let mut normalized_codes = Vec::new();
    let mut seen_codes = HashSet::new();

    for permission_code in permission_codes {
        let code = permission_code.trim();
        if code.is_empty() {
            continue;
        }

        let Some(permission) = Permission::from_code(code) else {
            return Err(StaffServiceError::InvalidPermission.into());
        };

        let normalized_code = permission.code().to_string();
        if seen_codes.insert(normalized_code.clone()) {
            normalized_codes.push(normalized_code);
        }
    }

    Ok(normalized_codes)
}

// Persist permissions as a comma-separated string because the current role
// entity stores them in a single database column.
pub fn serialize_permission_codes(permission_codes: &[String]) -> Result<String, AppError> {
    Ok(normalize_permission_codes(permission_codes)?.join(","))
}

// Convert the stored comma-separated database value back into a list that
// can be attached to the authenticated staff context.
pub fn deserialize_permission_codes(serialized_permissions: &str) -> Vec<String> {
    serialized_permissions
        .split(',')
        .map(str::trim)
        .filter(|code| !code.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_permission_codes_trims_and_deduplicates_values() {
        let normalized_codes = normalize_permission_codes(&[
            " branch.create ".to_string(),
            "staff.invite".to_string(),
            "branch.create".to_string(),
        ])
        .unwrap();

        assert_eq!(
            normalized_codes,
            vec!["branch.create".to_string(), "staff.invite".to_string(),]
        );
    }

    #[test]
    fn has_all_permissions_requires_every_permission() {
        let permission_codes =
            build_permission_code_set(&["branch.create".to_string(), "staff.invite".to_string()]);

        assert!(has_all_permissions(
            &permission_codes,
            &[Permission::BranchCreate, Permission::StaffInvite],
        ));
        assert!(!has_all_permissions(
            &permission_codes,
            &[Permission::BranchCreate, Permission::StaffRoleCreate],
        ));
    }

    #[test]
    fn has_any_permission_accepts_any_matching_permission() {
        let permission_codes = build_permission_code_set(&["staff.invite".to_string()]);

        assert!(has_any_permission(
            &permission_codes,
            &[Permission::BranchCreate, Permission::StaffInvite],
        ));
        assert!(!has_any_permission(
            &permission_codes,
            &[Permission::BranchCreate, Permission::StaffInvitationRevoke,],
        ));
    }
}
