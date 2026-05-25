use crate::auth::permission::deserialize_permission_codes;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::entity::staff::{
    staff_entity::{self as Staff, StaffStatus},
    staff_role_entity as StaffRole,
};
use crate::entity::{OrganizationPrimaryId, UserPrimaryId};
use crate::errors::app_error::AppError;
use crate::errors::staff_service_errors::StaffServiceError;

pub struct AuthResolver;
pub struct ResolvedStaffAccess {
    pub staff: Staff::Model,
    pub role: StaffRole::Model,
    pub permission_codes: Vec<String>,
}

impl AuthResolver {
    pub async fn staff_access(
        db_transaction: &impl ConnectionTrait,
        user_id: UserPrimaryId,
        organization_id: OrganizationPrimaryId,
    ) -> Result<ResolvedStaffAccess, AppError> {
        let staff = Staff::Entity::find()
            .filter(Staff::Column::UserId.eq(user_id))
            .filter(Staff::Column::OrganizationId.eq(organization_id))
            .filter(Staff::Column::Status.eq(StaffStatus::Active))
            .one(db_transaction)
            .await?
            .ok_or(StaffServiceError::NotFound)?;
        let role = StaffRole::Entity::find_by_id(staff.role_id)
            .one(db_transaction)
            .await?
            .ok_or(StaffServiceError::RoleNotFound)?;

        Ok(ResolvedStaffAccess {
            permission_codes: deserialize_permission_codes(&role.permissions),
            staff,
            role,
        })
    }
}
