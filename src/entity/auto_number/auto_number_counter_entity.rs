use crate::entity::PrimaryId;
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "auto_number_counters")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: PrimaryId,
    #[sea_orm(unique_key = "auto_number_counter_period")]
    pub series_id: PrimaryId,
    #[sea_orm(string_len = 20)]
    #[sea_orm(unique_key = "auto_number_counter_period")]
    pub period_key: String,
    pub next_number: i64,
    pub last_issued_number: Option<i64>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type AutoNumberCounterModel = Model;
pub type AutoNumberCounterEntity = Entity;
