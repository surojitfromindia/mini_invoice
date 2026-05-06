use crate::entity::user_entity::{self as UserEntity, UserStatus};

use crate::errors::app_error::AppError;
use crate::errors::user_service_errors::UserServiceError;
use crate::service::service_context::ServiceContext;
use crate::service::user_credential_service::UserCredentialService;
use crate::utils::date_helpers::DateHelper;
use crate::utils::id_generator::IdGenerator;
use sea_orm::{ActiveModelTrait, Set, TransactionTrait};
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
    pub async fn create_user_account(
        ctx: &ServiceContext,
        payload: CreateUserAccount,
    ) -> Result<String, AppError> {
        let email = payload.email.clone();
        // check is email exists.
        if UserService::check_email_is_registered(ctx, &email).await? {
            return Err(UserServiceError::EmailAlreadyExists.into());
        }
        let user = Self::prepare_user(payload.first_name, payload.last_name, payload.email);
        ctx.app_state.primary_write_replica.transaction::<_,(), AppError>(|txn|{
            Box::pin(async move {
                // insert data with transaction
                let user: UserEntity::Model = user.insert(txn).await?;
                UserCredentialService::save_credential(txn, user.id, payload.password).await?;
                Ok(())
            })
        }).await?;
        Ok(email)
    }
    

    fn prepare_user(first_name: String, last_name:String, email: String)-> UserEntity::ActiveModel{
        let public_id = IdGenerator::get_user_id();
        let now = DateHelper::now().value();
        UserEntity::ActiveModel {
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
        UserEntity::Entity::find_by_email(email.into())
            .one(&ctx.app_state.primary_read_replica)
            .await
            .map(|x| x.is_some())
            .map_err(AppError::DatabaseError)
    }
}
