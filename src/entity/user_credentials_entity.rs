use crate::entity::PrimaryId;
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "user_credentials_status"
)]
pub enum UserCredentialsStatus {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "deleted")]
    Deleted,
}

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_credentials")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: PrimaryId,
    #[sea_orm(column_type = "Text")]
    pub password_hash: String,
    #[sea_orm(column_type = "Text", nullable)]
    pub refresh_token_hash: Option<String>,
    pub failed_attempts: i16,
    pub status: UserCredentialsStatus,
    pub created_at: DateTimeUtc,
    pub password_changed_at: Option<DateTimeUtc>,
    pub last_login_at: Option<DateTimeUtc>,
    pub refresh_token_expires_at: Option<DateTimeUtc>,
}
impl ActiveModelBehavior for ActiveModel {}

pub type UserCredentialsModel = Model;
pub type UserCredentialsEntity = Entity;
