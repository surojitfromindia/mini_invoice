use crate::entity::PublicId;
use sea_orm::entity::prelude::*;

#[derive(Debug, Default, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "client_app_status")]
pub enum ClientAppStatus {
    #[sea_orm(string_value = "active")]
    #[default]
    Active,
    #[sea_orm(string_value = "deleted")]
    Deleted,
}

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "client_apps")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub public_id: PublicId,
    pub name: String,
    #[sea_orm(unique)]
    pub client_secret: String,
    pub status: ClientAppStatus,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type ClientAppModel = Model;
pub type ClientAppEntity = Entity;
