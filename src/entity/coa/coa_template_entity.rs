use crate::entity::{GenericStatus, PrimaryId, PublicId};
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "coa_templates")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: PrimaryId,
    #[sea_orm(unique)]
    pub public_id: PublicId,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub description: Option<String>,
    pub country_iso_code: String,
    pub accounting_standard: Option<String>,
    pub is_default: bool,
    pub organization_id: PrimaryId,
    pub status: GenericStatus,
    pub created_by_actor_id: PrimaryId,
    pub updated_by_actor_id: Option<PrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type CoaTemplateModel = Model;
pub type CoaTemplateEntity = Entity;
