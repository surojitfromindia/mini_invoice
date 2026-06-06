use crate::config::settings::Settings;
use crate::entity::PrimaryId;
use crate::entity::user_credentials_entity::{self as UserCredentials, UserCredentialsStatus};
use crate::errors::app_error::AppError;
use crate::errors::user_credential_service_errors::UserCredentialServiceError;
use crate::service::service_context::ServiceContext;
use crate::utils::date_helpers::DateHelper;
use crate::utils::password_helpers::PasswordHelpers;
use sea_orm::prelude::Expr;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseTransaction, DbErr, EntityTrait,
    ExprTrait, QueryFilter, Set,
};

pub struct UserCredentialService;

pub type UserCredentialsModel = UserCredentials::Model;

impl UserCredentialService {
    pub async fn save_credential(
        settings: &Settings,
        db_transaction: &DatabaseTransaction,
        user_id: PrimaryId,
        plain_password: impl Into<&String>,
    ) -> Result<bool, AppError> {
        let now = DateHelper::now().value();
        let password_hash = PasswordHelpers::hash_login_password(settings, plain_password.into())?;

        let user_credential_active_model = UserCredentials::ActiveModel {
            status: Set(UserCredentialsStatus::Active),
            user_id: Set(user_id),
            password_hash: Set(password_hash),
            refresh_token_hash: Set(None),
            failed_attempts: Set(0),
            created_at: Set(now),
            password_changed_at: Set(None),
            last_login_at: Set(None),
            refresh_token_expires_at: Set(None),
            ..Default::default()
        };
        user_credential_active_model.insert(db_transaction).await?;
        Ok(true)
    }

    pub async fn save_last_login_at(
        db_transaction: &impl ConnectionTrait,
        user_id: PrimaryId,
    ) -> Result<(), DbErr> {
        let now = DateHelper::now().value();

        UserCredentials::Entity::update_many()
            .col_expr(UserCredentials::COLUMN.last_login_at, Expr::value(now))
            .filter(UserCredentials::COLUMN.user_id.eq(user_id))
            .exec(db_transaction)
            .await?;

        Ok(())
    }

    pub async fn reset_failed_attempts(
        db_transaction: &impl ConnectionTrait,
        user_id: PrimaryId,
    ) -> Result<(), DbErr> {
        UserCredentials::Entity::update_many()
            .col_expr(UserCredentials::COLUMN.failed_attempts, Expr::value(0))
            .filter(UserCredentials::COLUMN.user_id.eq(user_id))
            .exec(db_transaction)
            .await?;
        Ok(())
    }

    pub async fn inc_failed_attempts(
        db_transaction: &impl ConnectionTrait,
        user_id: PrimaryId,
    ) -> Result<(), DbErr> {
        UserCredentials::Entity::update_many()
            .col_expr(
                UserCredentials::COLUMN.failed_attempts,
                Expr::col(UserCredentials::COLUMN.failed_attempts).add(1),
            )
            .filter(UserCredentials::COLUMN.user_id.eq(user_id))
            .exec(db_transaction)
            .await?;
        Ok(())
    }

    pub async fn get_credential(
        ctx: &ServiceContext,
        user_id: PrimaryId,
    ) -> Result<UserCredentialsModel, AppError> {
        let data = UserCredentials::Entity::find()
            .filter(UserCredentials::COLUMN.user_id.eq(user_id))
            .one(&ctx.app_state.primary_read_replica)
            .await?
            .ok_or(UserCredentialServiceError::CredentialNotFound)?;
        Ok(data)
    }

    pub async fn credential_exists(
        db_transaction: &impl ConnectionTrait,
        user_id: PrimaryId,
    ) -> Result<bool, AppError> {
        let exists = UserCredentials::Entity::find()
            .filter(UserCredentials::COLUMN.user_id.eq(user_id))
            .one(db_transaction)
            .await?
            .is_some();
        Ok(exists)
    }

    pub async fn verify_login_password(
        settings: &Settings,
        plain_password: &str,
        credential: &UserCredentialsModel,
    ) -> Result<(), AppError> {
        let is_match = PasswordHelpers::verify_login_password(
            settings,
            plain_password,
            &credential.password_hash,
        )?;

        if !is_match {
            return Err(AppError::InvalidCredentials);
        }
        Ok(())
    }

    pub async fn save_refresh_token(
        settings: &Settings,
        db_transaction: &impl ConnectionTrait,
        user_id: PrimaryId,
        refresh_token: &str,
        refresh_token_expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), AppError> {
        let refresh_token_hash = PasswordHelpers::hash_secret(settings, refresh_token)?;

        UserCredentials::Entity::update_many()
            .col_expr(
                UserCredentials::COLUMN.refresh_token_hash,
                Expr::value(refresh_token_hash),
            )
            .col_expr(
                UserCredentials::COLUMN.refresh_token_expires_at,
                Expr::value(refresh_token_expires_at),
            )
            .filter(UserCredentials::COLUMN.user_id.eq(user_id))
            .exec(db_transaction)
            .await?;
        Ok(())
    }

    pub async fn clear_refresh_token(
        db_transaction: &impl ConnectionTrait,
        user_id: PrimaryId,
    ) -> Result<(), DbErr> {
        UserCredentials::Entity::update_many()
            .col_expr(
                UserCredentials::COLUMN.refresh_token_hash,
                Expr::value(None::<String>),
            )
            .col_expr(
                UserCredentials::COLUMN.refresh_token_expires_at,
                Expr::value(None::<chrono::DateTime<chrono::Utc>>),
            )
            .filter(UserCredentials::COLUMN.user_id.eq(user_id))
            .exec(db_transaction)
            .await?;
        Ok(())
    }

    pub fn verify_refresh_token(
        settings: &Settings,
        plain_refresh_token: &str,
        credential: &UserCredentialsModel,
    ) -> Result<bool, AppError> {
        let Some(refresh_token_hash) = &credential.refresh_token_hash else {
            return Ok(false);
        };
        PasswordHelpers::verify_secret(settings, plain_refresh_token, refresh_token_hash)
    }
}
