use crate::config::settings::Settings;
use crate::entity::user_credentials_entity::{self as UserCredentials};
use crate::errors::app_error::AppError;
use crate::errors::user_credential_service_errors::UserCredentialServiceError;
use crate::service::service_context::ServiceContext;
use crate::utils::date_helpers::DateHelper;
use crate::utils::password_helpers::PasswordHelpers;
use sea_orm::{ActiveModelTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};

pub struct UserCredentialService;

pub type UserCredentialsModel = UserCredentials::Model;

impl UserCredentialService {
    pub async fn save_credential(
        settings: &Settings,
        db_transaction: &DatabaseTransaction,
        user_id: i32,
        plain_password: impl Into<&String>,
    ) -> Result<bool, AppError> {
        let now = DateHelper::now().value();
        let password_hash = PasswordHelpers::hash_login_password(settings, plain_password.into())?;

        let user_credential = UserCredentials::ActiveModel {
            user_id: Set(user_id),
            password_hash: Set(password_hash),
            failed_attempts: Set(0),
            created_at: Set(now),
            password_changed_at: Set(None),
            last_login_at: Set(None),
            ..Default::default()
        };
        user_credential
            .insert(db_transaction)
            .await
            .map_err(AppError::DatabaseError)?;
        Ok(true)
    }

    pub async fn get_credential(
        ctx: &ServiceContext,
        user_id: i32,
    ) -> Result<UserCredentialsModel, AppError> {
        let data = UserCredentials::Entity::find()
            .filter(UserCredentials::COLUMN.user_id.eq(user_id))
            .one(&ctx.app_state.primary_read_replica)
            .await
            .map_err(AppError::DatabaseError)?
            .ok_or(UserCredentialServiceError::CredentialNotFound)?;
        Ok(data)
    }
}
