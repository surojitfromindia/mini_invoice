use crate::entity::{PrimaryId, PublicId};
use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "party_type")]
pub enum PartyType {
    #[sea_orm(string_value = "customer")]
    Customer,
    #[sea_orm(string_value = "vendor")]
    Vendor,
    #[sea_orm(string_value = "both")]
    Both,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "party_kind")]
pub enum PartyKind {
    #[sea_orm(string_value = "person")]
    Person,
    #[sea_orm(string_value = "business")]
    Business,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "party_status")]
pub enum PartyStatus {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "inactive")]
    Inactive,
    #[sea_orm(string_value = "deleted")]
    Deleted,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "party_source")]
pub enum PartySource {
    #[sea_orm(string_value = "manual")]
    Manual,
    #[sea_orm(string_value = "pos_quick")]
    PosQuick,
    #[sea_orm(string_value = "import")]
    Import,
}

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "parties")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: PrimaryId,
    #[sea_orm(unique_key = "org_party_code", indexed)]
    pub organization_id: PrimaryId,
    #[sea_orm(unique)]
    pub public_id: PublicId,
    #[sea_orm(unique_key = "org_party_code")]
    pub code: String,
    #[sea_orm(indexed)]
    pub party_type: PartyType,
    pub party_kind: PartyKind,
    #[sea_orm(indexed)]
    pub status: PartyStatus,
    pub source: PartySource,
    #[sea_orm(indexed)]
    pub display_name: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub legal_name: Option<String>,
    #[sea_orm(indexed)]
    pub phone: Option<String>,
    pub email: Option<String>,
    #[sea_orm(indexed)]
    pub tax_no: Option<String>,
    pub tax_treatment: Option<String>,
    pub country_iso_code: Option<String>,
    pub currency_iso_code: Option<String>,
    pub payment_terms_days: Option<i16>,
    pub credit_limit: Option<Decimal>,
    pub allow_credit: bool,
    pub notes: Option<String>,
    pub created_by_actor_id: PrimaryId,
    pub updated_by_actor_id: Option<PrimaryId>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

impl ActiveModelBehavior for ActiveModel {}

pub type PartyModel = Model;
pub type PartyEntity = Entity;
