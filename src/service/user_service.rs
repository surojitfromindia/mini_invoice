use crate::entity::user_entity::{self as User, UserStatus};

use crate::entity::{PublicId, UserPrimaryId};
use crate::errors::app_error::AppError;
use crate::errors::user_service_errors::UserServiceError;
use crate::service::actor_service::ActorService;
use crate::service::service_context::ServiceContext;
use crate::service::user_credential_service::UserCredentialService;
use crate::utils::date_helpers::DateHelper;
use crate::utils::id_generator::IdGenerator;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set, TransactionTrait};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateUserAccount {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
}

pub struct UserService;

impl UserService {
    pub async fn get_or_create_user_without_password(
        db_transaction: &impl sea_orm::ConnectionTrait,
        first_name: String,
        last_name: String,
        email: String,
    ) -> Result<User::Model, AppError> {
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
        payload: CreateUserAccount,
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
    ) -> Result<(i32, PublicId), AppError> {
        let user = User::Entity::find_by_email(email.clone())
            .one(&ctx.app_state.primary_read_replica)
            .await
            .map_err(AppError::Database)?
            .ok_or(UserServiceError::NotFound)?;
        Ok((user.id, user.public_id))
    }

    pub async fn get_user_by_id(
        ctx: &ServiceContext,
        id: UserPrimaryId,
    ) -> Result<User::Model, AppError> {
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
        let user = User::Entity::find()
            .filter(User::COLUMN.public_id.eq(public_id.clone()))
            .one(&ctx.app_state.primary_read_replica)
            .await
            .map_err(AppError::Database)?
            .ok_or(UserServiceError::NotFound)?;
        Ok(user)
    }
}
