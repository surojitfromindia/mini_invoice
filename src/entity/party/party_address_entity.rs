use crate::entity::PrimaryId;
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "party_address_type")]
pub enum PartyAddressType {
    #[sea_orm(string_value = "billing")]
    Billing,
    #[sea_orm(string_value = "shipping")]
    Shipping,
    #[sea_orm(string_value = "registered")]
    Registered,
    #[sea_orm(string_value = "office")]
    Office,
    #[sea_orm(string_value = "home")]
    Home,
    #[sea_orm(string_value = "other")]
    Other,
}

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "party_addresses")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: PrimaryId,
    #[sea_orm(indexed)]
    pub party_id: PrimaryId,
    pub address_type: PartyAddressType,
    pub label: Option<String>,
    pub line1: String,
    pub line2: Option<String>,
    pub city: Option<String>,
    pub state_region: Option<String>,
    pub postal_code: Option<String>,
    pub country_iso_code: Option<String>,
    pub is_default_billing: bool,
    pub is_default_shipping: bool,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type PartyAddressModel = Model;
pub type PartyAddressEntity = Entity;
