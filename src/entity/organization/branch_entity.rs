use crate::entity::{PrimaryId, PublicId};
use sea_orm::entity::prelude::*;

// Branches need an inactive lifecycle state in addition to soft deletion so the
// organization can temporarily stop using a branch without losing its history.
#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "branch_status")]
pub enum BranchStatus {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "inactive")]
    Inactive,
    #[sea_orm(string_value = "deleted")]
    Deleted,
}

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "branches")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: PrimaryId,
    pub organization_id: PrimaryId,
    #[sea_orm(unique)]
    pub public_id: PublicId,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub is_primary: bool,
    pub status: BranchStatus,
    pub created_by_actor_id: PrimaryId,
    pub updated_by_actor_id: Option<PrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type BranchModel = Model;
pub type BranchEntity = Entity;
