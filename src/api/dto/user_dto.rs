use serde::{Deserialize, Serialize};

use super::common_dto::IntoServiceInput;
use crate::service::user_service::{
    CreateUserAccountInput, CurrentUserOrganization, CurrentUserProfile,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserAccountRequestDto {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
}

impl IntoServiceInput<CreateUserAccountInput> for CreateUserAccountRequestDto {
    fn into_service_input(self) -> CreateUserAccountInput {
        CreateUserAccountInput {
            first_name: self.first_name,
            last_name: self.last_name,
            email: self.email,
            password: self.password,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAccountCreatedResponseDto {
    pub email: String,
}

impl UserAccountCreatedResponseDto {
    pub fn from_service_output(email: String) -> Self {
        Self { email }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentUserOrganizationResponseDto {
    pub public_id: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub country_iso_code: String,
    pub currency_iso_code: String,
}

impl From<CurrentUserOrganization> for CurrentUserOrganizationResponseDto {
    fn from(value: CurrentUserOrganization) -> Self {
        Self {
            public_id: value.public_id,
            name_primary: value.name_primary,
            name_secondary: value.name_secondary,
            country_iso_code: value.country_iso_code,
            currency_iso_code: value.currency_iso_code,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentUserResponseDto {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub has_organization: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<CurrentUserOrganizationResponseDto>,
}

impl CurrentUserResponseDto {
    pub fn from_service_output(value: CurrentUserProfile) -> Self {
        let organization = value
            .organization
            .map(CurrentUserOrganizationResponseDto::from);

        Self {
            email: value.email,
            first_name: value.first_name,
            last_name: value.last_name,
            has_organization: organization.is_some(),
            organization,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_user_response_serializes_camel_case_keys() {
        let response = CurrentUserResponseDto {
            email: "user@example.com".to_string(),
            first_name: "Mini".to_string(),
            last_name: "User".to_string(),
            has_organization: true,
            organization: Some(CurrentUserOrganizationResponseDto {
                public_id: "org_123".to_string(),
                name_primary: "Acme".to_string(),
                name_secondary: Some("Acme Trading".to_string()),
                country_iso_code: "IN".to_string(),
                currency_iso_code: "INR".to_string(),
            }),
        };

        let json = serde_json::to_value(response).unwrap();

        assert_eq!(
            json,
            serde_json::json!({
                "email": "user@example.com",
                "firstName": "Mini",
                "lastName": "User",
                "hasOrganization": true,
                "organization": {
                    "publicId": "org_123",
                    "namePrimary": "Acme",
                    "nameSecondary": "Acme Trading",
                    "countryIsoCode": "IN",
                    "currencyIsoCode": "INR"
                }
            })
        );
    }
}
