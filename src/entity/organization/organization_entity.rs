use sea_orm::entity::prelude::*;
use crate::entity::{PublicId, ActorPrimaryId};

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "organizations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub user_id: Option<i32>,
    #[sea_orm(unique)]
    pub public_id: Option<PublicId>,
    pub name_primary : String,
    pub name_secondary : Option<String>,
    pub created_by_actor_id: ActorPrimaryId,
    pub updated_by_actor_id: Option<ActorPrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
impl ActiveModelBehavior for ActiveModel {}
