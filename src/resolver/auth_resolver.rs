use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::entity::organization::{organization_entity as Organization, staff_entity as Staff};
use crate::entity::{PublicId, UserPrimaryId};
use crate::errors::app_error::AppError;
use crate::errors::staff_service_errors::StaffServiceError;

use super::public_id_resolver::PublicIdResolver;

pub struct AuthResolver;

impl AuthResolver {
    pub async fn resolve_user_actor(
        db_transaction: &impl ConnectionTrait,
        user_public_id: &PublicId,
    ) -> Result<crate::entity::actor_entity::Model, AppError> {
        PublicIdResolver::user_actor(db_transaction, user_public_id).await
    }

    pub async fn resolve_user_organization(
        db_transaction: &impl ConnectionTrait,
        user_id: UserPrimaryId,
        organization_public_id: &str,
    ) -> Result<Organization::Model, AppError> {
        let organization =
            PublicIdResolver::organization(db_transaction, organization_public_id).await?;

        let membership = Staff::Entity::find()
            .filter(Staff::Column::UserId.eq(user_id))
            .filter(Staff::Column::OrganizationId.eq(organization.id))
            .one(db_transaction)
            .await?;

        membership
            .map(|_| organization)
            .ok_or(StaffServiceError::NotFound.into())
    }
}
