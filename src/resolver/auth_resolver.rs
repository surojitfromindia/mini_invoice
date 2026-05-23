use crate::auth::permission::deserialize_permission_codes;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::entity::organization::{
    staff_entity::{self as Staff, StaffStatus},
    staff_role_entity as StaffRole,
};
use crate::entity::{OrganizationPrimaryId, UserPrimaryId};
use crate::errors::app_error::AppError;
use crate::errors::staff_service_errors::StaffServiceError;

use super::public_id_resolver::PublicIdResolver;

pub struct AuthResolver;
pub struct ResolvedStaffAccess {
    pub staff: Staff::Model,
    pub role: StaffRole::Model,
    pub permission_codes: Vec<String>,
}

impl AuthResolver {
    pub async fn resolve_user_actor(
        db_transaction: &impl ConnectionTrait,
        user_public_id: &str,
    ) -> Result<crate::entity::actor_entity::Model, AppError> {
        PublicIdResolver::user_actor(db_transaction, user_public_id).await
    }

    pub async fn resolve_user_organization_membership(
        db_transaction: &impl ConnectionTrait,
        user_id: UserPrimaryId,
        organization_public_id: &str,
    ) -> Result<ResolvedStaffAccess, AppError> {
        let organization =
            PublicIdResolver::organization(db_transaction, organization_public_id).await?;
        Self::resolve_staff_membership_by_ids(db_transaction, user_id, organization.id).await
    }

    pub async fn resolve_staff_membership_by_ids(
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
