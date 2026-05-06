use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_credentials")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: i32,
    pub password_hash: String,
    pub failed_attempts: i16,
    pub created_at: DateTimeUtc,
    pub password_changed_at: Option<DateTimeUtc>,
    pub last_login_at: Option<DateTimeUtc>,
}
impl ActiveModelBehavior for ActiveModel {}
