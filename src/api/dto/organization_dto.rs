use serde::Deserialize;

use crate::service::organization_service::CreateOrganizationInput;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateOrganizationRequestDto {
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub country_iso_code: String,
    pub currency_iso_code: String,
}

impl CreateOrganizationRequestDto {
    pub fn into_service_input(self) -> CreateOrganizationInput {
        CreateOrganizationInput {
            name_primary: self.name_primary,
            name_secondary: self.name_secondary,
            country_iso_code: self.country_iso_code,
            currency_iso_code: self.currency_iso_code,
        }
    }
}
