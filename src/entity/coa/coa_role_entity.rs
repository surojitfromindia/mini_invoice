use crate::entity::{GenericStatus, PrimaryId};
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "coa_account_roles")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: PrimaryId,
    pub organization_id: PrimaryId,
    #[sea_orm(unique_key = "coa_template_role")]
    pub coa_template_id: PrimaryId,
    #[sea_orm(unique_key = "coa_template_role")]
    pub account_id: PrimaryId,
    #[sea_orm(unique_key = "coa_template_role")]
    pub role_slug: String,
    pub status: GenericStatus,
    pub created_by_actor_id: PrimaryId,
    pub updated_by_actor_id: Option<PrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type CoaAccountRoleModel = Model;
pub type CoaAccountRoleEntity = Entity;
