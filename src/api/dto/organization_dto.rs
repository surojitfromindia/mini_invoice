use serde::Deserialize;

use crate::service::organization_service::CreateOrganizationInput;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateOrganizationRequestDto {
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub country_iso_code: String,
    pub currency_iso_code: String,
}

impl From<CreateOrganizationRequestDto> for CreateOrganizationInput {
    fn from(value: CreateOrganizationRequestDto) -> Self {
        Self {
            name_primary: value.name_primary,
            name_secondary: value.name_secondary,
            country_iso_code: value.country_iso_code,
            currency_iso_code: value.currency_iso_code,
        }
    }
}
