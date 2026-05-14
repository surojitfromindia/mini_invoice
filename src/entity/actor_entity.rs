use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "actor_type")]
pub enum ActorType {
    #[sea_orm(string_value = "app")]
    App,
    #[sea_orm(string_value = "user")]
    User,
    #[sea_orm(string_value = "system")]
    System,
}


#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "actors")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: Option<i32>,
    pub public_user_id: Option<String>,
    pub app_id: Option<i32>,
    pub public_app_id: Option<String>,
    pub actor_type: ActorType,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}
