use crate::errors::app_error::AppError;
use sea_orm::{ActiveModelTrait, DatabaseTransaction, Set};

use crate::entity::user_credentials_entity as UserCredentialsEntity;
use crate::utils::date_helpers::DateHelper;
use crate::utils::password_helpers::PasswordHelpers;

pub struct UserCredentialService;

impl UserCredentialService {
    pub async fn save_credential(
        db_transaction: &DatabaseTransaction,
        user_id: i32,
        plain_password: impl Into<String>,
    ) -> Result<bool, AppError> {
        let now = DateHelper::now().value();
        let password_hash = PasswordHelpers::hash_plain_password(plain_password.into())?;

        let user_credential = UserCredentialsEntity::ActiveModel {
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
}
