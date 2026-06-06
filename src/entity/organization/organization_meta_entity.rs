use crate::entity::PrimaryId;
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "organization_meta_status"
)]
pub enum OrganizationMetaStatus {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "deleted")]
    Deleted,
}

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "organizations_meta")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub organization_id: PrimaryId,
    #[sea_orm(string_len = 2)]
    pub country_iso_code: String,
    #[sea_orm(string_len = 3)]
    pub currency_iso_code: String,
    pub default_branch_id: Option<PrimaryId>,
    pub status: OrganizationMetaStatus,
    pub updated_by_actor_id: Option<PrimaryId>,
    pub created_by_actor_id: PrimaryId,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}
impl ActiveModelBehavior for ActiveModel {}

pub type OrganizationMetaModel = Model;
pub type OrganizationMetaEntity = Entity;
