use crate::entity::organization::{
    organization_entity as Organization, organization_meta_entity as OrganizationMeta,
};
use crate::entity::staff::staff_entity::{self as Staff, StaffStatus};
use crate::entity::user_entity::{self as User, UserModel, UserStatus};

use crate::entity::{PrimaryId, PublicId};
use crate::errors::app_error::AppError;
use crate::errors::organization_service_errors::OrgServiceError;
use crate::errors::user_service_errors::UserServiceError;
use crate::service::actor_service::ActorService;
use crate::service::service_context::ServiceContext;
use crate::service::user_credential_service::UserCredentialService;
use crate::utils::date_helpers::DateHelper;
use crate::utils::id_generator::IdGenerator;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};

pub struct CreateUserAccountInput {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
}

pub struct CurrentUserOrganization {
    pub public_id: PublicId,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub country_iso_code: String,
    pub currency_iso_code: String,
}

pub struct CurrentUserProfile {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub organization: Option<CurrentUserOrganization>,
}

pub struct UserService;

impl UserService {
    pub async fn get_user_by_public_id_from_db(
        db_transaction: &impl sea_orm::ConnectionTrait,
        public_id: &PublicId,
    ) -> Result<UserModel, AppError> {
        let user = User::Entity::find()
            .filter(User::COLUMN.public_id.eq(public_id.clone()))
            .one(db_transaction)
            .await
            .map_err(AppError::Database)?
            .ok_or(UserServiceError::NotFound)?;
        Ok(user)
    }

    pub async fn get_or_create_user_without_password(
        db_transaction: &impl sea_orm::ConnectionTrait,
        first_name: String,
        last_name: String,
        email: String,
    ) -> Result<UserModel, AppError> {
        let normalized_email = email.trim().to_lowercase();
        if let Some(existing_user) = User::Entity::find_by_email(normalized_email.clone())
            .one(db_transaction)
            .await?
        {
            return Ok(existing_user);
        }

        let user = Self::prepare_user(first_name, last_name, normalized_email);
        let user = user.insert(db_transaction).await?;
        ActorService::create_from_user(db_transaction, user.id, user.public_id.clone()).await?;
        Ok(user)
    }

    pub async fn create_user_account(
        ctx: &ServiceContext,
        payload: CreateUserAccountInput,
    ) -> Result<String, AppError> {
        let settings = ctx.app_state.settings.clone();
        let email = payload.email.trim().to_lowercase();
        // check is email exists.
        if UserService::check_email_is_registered(ctx, &email).await? {
            return Err(UserServiceError::EmailAlreadyExists.into());
        }
        let user = Self::prepare_user(payload.first_name, payload.last_name, email.clone());

        ctx.app_state
            .primary_write_replica
            .transaction::<_, (), AppError>(|txn| {
                Box::pin(async move {
                    // insert data with transaction
                    let user: User::Model = user.insert(txn).await?;
                    // save user credential
                    UserCredentialService::save_credential(
                        &settings,
                        txn,
                        user.id,
                        &payload.password,
                    )
                    .await?;
                    // create an actor for this user.
                    ActorService::create_from_user(txn, user.id, user.public_id).await?;
                    Ok(())
                })
            })
            .await?;
        Ok(email)
    }

    pub async fn get_current_user_profile(
        ctx: &ServiceContext,
    ) -> Result<CurrentUserProfile, AppError> {
        let user_id = ctx.get_user_id()?;
        let user = Self::get_user_by_id(ctx, user_id).await?;
        let organization = Self::get_default_organization_profile(ctx, user_id).await?;

        Ok(CurrentUserProfile {
            email: user.email,
            first_name: user.first_name,
            last_name: user.last_name,
            organization,
        })
    }

    async fn get_default_organization_profile(
        ctx: &ServiceContext,
        user_id: PrimaryId,
    ) -> Result<Option<CurrentUserOrganization>, AppError> {
        let staff = Staff::Entity::find()
            .filter(Staff::COLUMN.user_id.eq(user_id))
            .filter(Staff::COLUMN.status.eq(StaffStatus::Active))
            .filter(Staff::COLUMN.is_default_organization.eq(true))
            .one(&ctx.app_state.primary_read_replica)
            .await?;

        let Some(staff) = staff else {
            return Ok(None);
        };

        let organization = Organization::Entity::find_by_id(staff.organization_id)
            .one(&ctx.app_state.primary_read_replica)
            .await?
            .ok_or(OrgServiceError::NotFound)?;
        let organization_meta = OrganizationMeta::Entity::find_by_id(organization.id)
            .one(&ctx.app_state.primary_read_replica)
            .await?
            .ok_or(OrgServiceError::NotFound)?;

        Ok(Some(CurrentUserOrganization {
            public_id: organization.public_id,
            name_primary: organization.name_primary,
            name_secondary: organization.name_secondary,
            country_iso_code: organization_meta.country_iso_code,
            currency_iso_code: organization_meta.currency_iso_code,
        }))
    }

    fn prepare_user(first_name: String, last_name: String, email: String) -> User::ActiveModel {
        let public_id = IdGenerator::get_user_id();
        let now = DateHelper::now().value();
        User::ActiveModel {
            public_id: Set(public_id),
            first_name: Set(first_name),
            last_name: Set(last_name),
            email: Set(email),
            email_verified: Set(false),
            status: Set(UserStatus::Active),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
    }

    async fn check_email_is_registered(
        ctx: &ServiceContext,
        email: impl Into<String>,
    ) -> Result<bool, AppError> {
        User::Entity::find_by_email(email.into())
            .one(&ctx.app_state.primary_read_replica)
            .await
            .map(|x| x.is_some())
            .map_err(AppError::Database)
    }

    pub async fn get_user_id_by_email(
        ctx: &ServiceContext,
        email: &String,
    ) -> Result<(PrimaryId, PublicId), AppError> {
        let user = User::Entity::find_by_email(email.clone())
            .one(&ctx.app_state.primary_read_replica)
            .await
            .map_err(AppError::Database)?
            .ok_or(UserServiceError::NotFound)?;
        Ok((user.id, user.public_id))
    }

    pub async fn get_user_by_id(
        ctx: &ServiceContext,
        id: PrimaryId,
    ) -> Result<UserModel, AppError> {
        let user = User::Entity::find_by_id(id)
            .one(&ctx.app_state.primary_read_replica)
            .await
            .map_err(AppError::Database)?
            .ok_or(UserServiceError::NotFound)?;
        Ok(user)
    }

    pub async fn get_user_by_public_id(
        ctx: &ServiceContext,
        public_id: &PublicId,
    ) -> Result<User::Model, AppError> {
        Self::get_user_by_public_id_from_db(&ctx.app_state.primary_read_replica, public_id).await
    }
}
