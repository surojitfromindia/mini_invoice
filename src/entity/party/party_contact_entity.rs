use crate::entity::PrimaryId;
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "party_contacts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: PrimaryId,
    #[sea_orm(indexed)]
    pub party_id: PrimaryId,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub designation: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_primary: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type PartyContactModel = Model;
pub type PartyContactEntity = Entity;
