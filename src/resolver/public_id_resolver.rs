use sea_orm::ConnectionTrait;

use crate::entity::organization::{
    branch_entity as Branch, organization_entity as Organization, staff_entity as Staff,
    staff_invitation_entity as StaffInvitation, staff_role_entity as StaffRole,
};
use crate::entity::{PublicId, actor_entity as Actor, user_entity as User};
use crate::errors::app_error::AppError;
use crate::errors::organization_service_errors::OrgServiceError;
use crate::errors::staff_service_errors::StaffServiceError;
use crate::errors::user_service_errors::UserServiceError;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

pub struct PublicIdResolver;

impl PublicIdResolver {
    pub async fn user(
        db_transaction: &impl ConnectionTrait,
        public_id: &PublicId,
    ) -> Result<User::Model, AppError> {
        User::Entity::find()
            .filter(User::Column::PublicId.eq(public_id.clone()))
            .one(db_transaction)
            .await
            .map_err(AppError::Database)?
            .ok_or(UserServiceError::NotFound.into())
    }

    pub async fn organization(
        db_transaction: &impl ConnectionTrait,
        public_id: &str,
    ) -> Result<Organization::Model, AppError> {
        Organization::Entity::find()
            .filter(Organization::Column::PublicId.eq(public_id))
            .one(db_transaction)
            .await?
            .ok_or(OrgServiceError::NotFound.into())
    }

    pub async fn branch(
        db_transaction: &impl ConnectionTrait,
        public_id: &str,
    ) -> Result<Branch::Model, AppError> {
        Branch::Entity::find()
            .filter(Branch::Column::PublicId.eq(public_id))
            .one(db_transaction)
            .await?
            .ok_or(OrgServiceError::BranchNotFound.into())
    }

    pub async fn staff(
        db_transaction: &impl ConnectionTrait,
        public_id: &PublicId,
    ) -> Result<Staff::Model, AppError> {
        Staff::Entity::find()
            .filter(Staff::Column::PublicId.eq(public_id.clone()))
            .one(db_transaction)
            .await?
            .ok_or(StaffServiceError::NotFound.into())
    }

    pub async fn staff_invitation(
        db_transaction: &impl ConnectionTrait,
        public_id: &PublicId,
    ) -> Result<StaffInvitation::Model, AppError> {
        StaffInvitation::Entity::find()
            .filter(StaffInvitation::Column::PublicId.eq(public_id.clone()))
            .one(db_transaction)
            .await?
            .ok_or(StaffServiceError::InvitationNotFound.into())
    }

    pub async fn staff_role(
        db_transaction: &impl ConnectionTrait,
        public_id: &str,
    ) -> Result<StaffRole::Model, AppError> {
        StaffRole::Entity::find()
            .filter(StaffRole::Column::PublicId.eq(public_id))
            .one(db_transaction)
            .await?
            .ok_or(StaffServiceError::RoleNotFound.into())
    }

    pub async fn user_actor(
        db_transaction: &impl ConnectionTrait,
        public_id: &PublicId,
    ) -> Result<Actor::Model, AppError> {
        Actor::Entity::find()
            .filter(Actor::Column::PublicUserId.eq(public_id))
            .one(db_transaction)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::ActorIdNotFound)
    }

    pub async fn client_app_actor(
        db_transaction: &impl ConnectionTrait,
        public_id: &PublicId,
    ) -> Result<Actor::Model, AppError> {
        Actor::Entity::find()
            .filter(Actor::Column::PublicClientAppId.eq(public_id))
            .one(db_transaction)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::ActorIdNotFound)
    }
}
