use super::common_types::{PrimaryId, PublicId};
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "actor_type")]
pub enum ActorType {
    #[sea_orm(string_value = "client_app")]
    ClientApp,
    #[sea_orm(string_value = "user")]
    User,
    #[sea_orm(string_value = "system")]
    System,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "actor_status")]
pub enum ActorStatus {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "deleted")]
    Deleted,
}

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "actors")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: PrimaryId,
    pub user_id: Option<PrimaryId>,
    pub public_user_id: Option<PublicId>,
    pub client_app_id: Option<PrimaryId>,
    pub public_client_app_id: Option<PublicId>,
    pub actor_type: ActorType,
    pub status: ActorStatus,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type ActorModel = Model;
pub type ActorEntity = Entity;
