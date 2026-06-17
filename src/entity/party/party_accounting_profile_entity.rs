use crate::entity::PrimaryId;
use sea_orm::entity::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "party_accounting_profiles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub party_id: PrimaryId,
    #[sea_orm(indexed)]
    pub organization_id: PrimaryId,
    pub default_sales_account_id: Option<PrimaryId>,
    pub default_purchase_account_id: Option<PrimaryId>,
    pub default_receivable_account_id: Option<PrimaryId>,
    pub default_payable_account_id: Option<PrimaryId>,
    pub default_output_tax_account_id: Option<PrimaryId>,
    pub default_input_tax_account_id: Option<PrimaryId>,
    pub created_by_actor_id: PrimaryId,
    pub updated_by_actor_id: Option<PrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type PartyAccountingProfileModel = Model;
pub type PartyAccountingProfileEntity = Entity;
