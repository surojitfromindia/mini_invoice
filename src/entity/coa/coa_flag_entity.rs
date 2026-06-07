use crate::entity::{GenericStatus, PrimaryId};
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "flag_scope")]
pub enum FlagScope {
    #[sea_orm(string_value = "self_only")]
    SelfOnly,
    #[sea_orm(string_value = "self_and_children")]
    SelfAndChildren,
    #[sea_orm(string_value = "children_only")]
    ChildrenOnly,
}

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "coa_account_flags")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: PrimaryId,
    pub organization_id: PrimaryId,
    #[sea_orm(unique_key = "coa_template_flag_root", indexed)]
    pub coa_template_id: PrimaryId,
    #[sea_orm(unique_key = "coa_template_flag_root", indexed)]
    pub root_account_id: PrimaryId,
    #[sea_orm(unique_key = "coa_template_flag_root", indexed)]
    pub flag_slug: String,
    pub scope: FlagScope,
    pub posting_accounts_only: bool,
    pub status: GenericStatus,
    pub created_by_actor_id: PrimaryId,
    pub updated_by_actor_id: Option<PrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type CoaAccountFlagModel = Model;
pub type CoaAccountFlagEntity = Entity;
