use sea_orm::entity::prelude::*;

pub type PublicId = String;
pub type PrimaryId = i32;

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "generic_status")]
pub enum GenericStatus {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "deleted")]
    Deleted,
}
