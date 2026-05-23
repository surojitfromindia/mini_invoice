use crate::errors::app_error::AppError;
use crate::errors::staff_service_errors::StaffServiceError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

pub fn has_permission(permission_codes: &[String], permission: Permission) -> bool {
    permission_codes
        .iter()
        .any(|code| code.as_str() == permission.code())
}

pub fn normalize_permission_codes(permission_codes: &[String]) -> Result<Vec<String>, AppError> {
    let mut normalized_codes = Vec::new();

    for permission_code in permission_codes {
        let code = permission_code.trim();
        if code.is_empty() {
            continue;
        }

        let Some(permission) = Permission::from_code(code) else {
            return Err(StaffServiceError::InvalidPermission.into());
        };

        let normalized_code = permission.code().to_string();
        if !normalized_codes.contains(&normalized_code) {
            normalized_codes.push(normalized_code);
        }
    }

    Ok(normalized_codes)
}

pub fn serialize_permission_codes(permission_codes: &[String]) -> Result<String, AppError> {
    Ok(normalize_permission_codes(permission_codes)?.join(","))
}

pub fn deserialize_permission_codes(serialized_permissions: &str) -> Vec<String> {
    serialized_permissions
        .split(',')
        .map(str::trim)
        .filter(|code| !code.is_empty())
        .map(str::to_string)
        .collect()
}
