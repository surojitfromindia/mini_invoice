use crate::entity::{GenericStatus, PrimaryId, PublicId};
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "chart_of_accounts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: PrimaryId,
    #[sea_orm(unique)]
    pub public_id: PublicId,
    pub organization_id: PrimaryId,
    #[sea_orm(unique_key = "coa_template_code")]
    pub coa_template_id: PrimaryId,
    pub parent_account_id: Option<PrimaryId>,
    pub account_group_id: Option<PrimaryId>,
    pub account_type_id: Option<PrimaryId>,
    #[sea_orm(unique_key = "coa_template_code")]
    pub code: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub description: Option<String>,
    pub level_no: i16,
    pub is_posting: bool,
    pub is_system_account: bool,
    pub status: GenericStatus,
    pub created_by_actor_id: PrimaryId,
    pub updated_by_actor_id: Option<PrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type ChartOfAccountModel = Model;
pub type ChartOfAccountEntity = Entity;
