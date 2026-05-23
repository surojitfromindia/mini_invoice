use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "sign_in_log_event_type"
)]
pub enum SignInLogEventType {
    #[sea_orm(string_value = "login_success")]
    LoginSuccess,
    #[sea_orm(string_value = "login_failure")]
    LoginFailure,
    #[sea_orm(string_value = "logout")]
    Logout,
    #[sea_orm(string_value = "refresh_token")]
    RefreshToken,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, FromJsonQueryResult)]
pub struct RequestContext {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device: Option<String>,
    pub os: Option<String>,
    pub browser: Option<String>,
}

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "login_logs")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: Option<i32>,
    pub identifier: String, // email string.
    pub created_at: DateTimeUtc,
    pub event_type: SignInLogEventType,
    pub request_context: RequestContext,
}
impl ActiveModelBehavior for ActiveModel {}

pub type LoginLogModel = Model;
pub type LoginLogEntity = Entity;
