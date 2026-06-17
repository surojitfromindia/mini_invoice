use crate::entity::{PrimaryId, PublicId};
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "auto_number_allocation_status"
)]
pub enum AutoNumberAllocationStatus {
    #[sea_orm(string_value = "committed")]
    Committed,
    #[sea_orm(string_value = "voided")]
    Voided,
}

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "auto_number_allocations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: PrimaryId,
    #[sea_orm(unique)]
    pub public_id: PublicId,
    #[sea_orm(unique_key = "auto_number_formatted_number")]
    pub organization_id: PrimaryId,
    pub branch_id: PrimaryId,
    pub series_id: PrimaryId,
    #[sea_orm(string_len = 80)]
    pub series_key: String,
    #[sea_orm(string_len = 20)]
    pub period_key: String,
    pub sequence_number: i64,
    #[sea_orm(string_len = 120)]
    #[sea_orm(unique_key = "auto_number_formatted_number")]
    pub formatted_number: String,
    #[sea_orm(nullable)]
    pub target_public_id: Option<PublicId>,
    pub status: AutoNumberAllocationStatus,
    pub created_by_actor_id: PrimaryId,
    pub created_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type AutoNumberAllocationModel = Model;
pub type AutoNumberAllocationEntity = Entity;
