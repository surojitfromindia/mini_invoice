use crate::entity::{PrimaryId, PublicId};
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "auto_number_reset_policy"
)]
pub enum AutoNumberResetPolicy {
    #[sea_orm(string_value = "never")]
    Never,
    #[sea_orm(string_value = "monthly")]
    Monthly,
    #[sea_orm(string_value = "calendar_year")]
    CalendarYear,
    #[sea_orm(string_value = "fiscal_year")]
    FiscalYear,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "auto_number_status")]
pub enum AutoNumberStatus {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "inactive")]
    Inactive,
    #[sea_orm(string_value = "deleted")]
    Deleted,
}

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "auto_number_series")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: PrimaryId,
    #[sea_orm(unique_key = "auto_number_series_scope")]
    pub organization_id: PrimaryId,
    #[sea_orm(unique_key = "auto_number_series_scope")]
    pub branch_id: PrimaryId,
    #[sea_orm(string_len = 80)]
    #[sea_orm(unique_key = "auto_number_series_scope")]
    pub series_key: String,
    #[sea_orm(unique)]
    pub public_id: PublicId,
    #[sea_orm(string_len = 80)]
    pub prefix_template: String,
    #[sea_orm(string_len = 40, nullable)]
    pub suffix_template: Option<String>,
    pub padding_width: i16,
    pub start_number: i64,
    pub increment_by: i16,
    pub reset_policy: AutoNumberResetPolicy,
    pub status: AutoNumberStatus,
    pub created_by_actor_id: PrimaryId,
    pub updated_by_actor_id: Option<PrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type AutoNumberSeriesModel = Model;
pub type AutoNumberSeriesEntity = Entity;
