use crate::db::listing::PageListResult;
use crate::entity::party::party_address_entity::PartyAddressType;
use crate::entity::party::party_entity::{PartyKind, PartySource, PartyStatus, PartyType};
use crate::service::party_service::{
    CreatePartyAccountingProfileInput, CreatePartyAddressInput, CreatePartyContactInput,
    PartyAccountingProfileDetail, PartyAddressDetail, PartyContactDetail, PartyDetail,
    PartyListItem, PartyListPageInput, PartySortField, SortDirection,
};
use schemars::JsonSchema;
use sea_orm::entity::prelude::Decimal;
use serde::{Deserialize, Serialize};

use super::common_dto::{IntoServiceInput, PagePaginationQuery};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum PartyTypeDto {
    Customer,
    Vendor,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum PartyKindDto {
    Person,
    Business,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum PartyStatusDto {
    Active,
    Inactive,
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum PartySourceDto {
    Manual,
    PosQuick,
    Import,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum PartyAddressTypeDto {
    Billing,
    Shipping,
    Registered,
    Office,
    Home,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum PartySortFieldDto {
    CreatedAt,
    DisplayName,
    Code,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum SortDirectionDto {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatePartyAddressRequestDto {
    pub address_type: PartyAddressTypeDto,
    pub label: Option<String>,
    pub line1: String,
    pub line2: Option<String>,
    pub city: Option<String>,
    pub state_region: Option<String>,
    pub postal_code: Option<String>,
    pub country_iso_code: Option<String>,
    pub is_default_billing: bool,
    pub is_default_shipping: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatePartyContactRequestDto {
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub designation: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_primary: bool,
}

#[derive(Debug, Clone, PartialEq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatePartyAccountingProfileRequestDto {
    pub default_sales_account_public_id: Option<String>,
    pub default_purchase_account_public_id: Option<String>,
    pub default_receivable_account_public_id: Option<String>,
    pub default_payable_account_public_id: Option<String>,
    pub default_output_tax_account_public_id: Option<String>,
    pub default_input_tax_account_public_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatePartyRequestDto {
    pub branch_public_id: Option<String>,
    pub party_type: PartyTypeDto,
    pub party_kind: PartyKindDto,
    pub status: Option<PartyStatusDto>,
    pub source: Option<PartySourceDto>,
    pub display_name: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub legal_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub tax_no: Option<String>,
    pub tax_treatment: Option<String>,
    pub country_iso_code: Option<String>,
    pub currency_iso_code: Option<String>,
    pub payment_terms_days: Option<i16>,
    pub credit_limit: Option<Decimal>,
    pub allow_credit: bool,
    pub notes: Option<String>,
    #[serde(default)]
    pub addresses: Vec<CreatePartyAddressRequestDto>,
    #[serde(default)]
    pub contacts: Vec<CreatePartyContactRequestDto>,
    pub accounting_profile: Option<CreatePartyAccountingProfileRequestDto>,
}

pub struct CreatePartyResolutionInput {
    pub branch_public_id: Option<String>,
    pub party_type: PartyTypeDto,
    pub party_kind: PartyKindDto,
    pub status: Option<PartyStatusDto>,
    pub source: Option<PartySourceDto>,
    pub display_name: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub legal_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub tax_no: Option<String>,
    pub tax_treatment: Option<String>,
    pub country_iso_code: Option<String>,
    pub currency_iso_code: Option<String>,
    pub payment_terms_days: Option<i16>,
    pub credit_limit: Option<Decimal>,
    pub allow_credit: bool,
    pub notes: Option<String>,
    pub addresses: Vec<CreatePartyAddressRequestDto>,
    pub contacts: Vec<CreatePartyContactRequestDto>,
    pub accounting_profile: Option<CreatePartyAccountingProfileRequestDto>,
}

pub struct CreatePartyAccountingProfileResolutionOutput {
    pub default_sales_account_id: Option<i32>,
    pub default_purchase_account_id: Option<i32>,
    pub default_receivable_account_id: Option<i32>,
    pub default_payable_account_id: Option<i32>,
    pub default_output_tax_account_id: Option<i32>,
    pub default_input_tax_account_id: Option<i32>,
}

impl CreatePartyRequestDto {
    pub fn into_resolution_input(self) -> CreatePartyResolutionInput {
        CreatePartyResolutionInput {
            branch_public_id: self.branch_public_id,
            party_type: self.party_type,
            party_kind: self.party_kind,
            status: self.status,
            source: self.source,
            display_name: self.display_name,
            name_primary: self.name_primary,
            name_secondary: self.name_secondary,
            legal_name: self.legal_name,
            phone: self.phone,
            email: self.email,
            tax_no: self.tax_no,
            tax_treatment: self.tax_treatment,
            country_iso_code: self.country_iso_code,
            currency_iso_code: self.currency_iso_code,
            payment_terms_days: self.payment_terms_days,
            credit_limit: self.credit_limit,
            allow_credit: self.allow_credit,
            notes: self.notes,
            addresses: self.addresses,
            contacts: self.contacts,
            accounting_profile: self.accounting_profile,
        }
    }
}

impl From<CreatePartyAccountingProfileResolutionOutput> for CreatePartyAccountingProfileInput {
    fn from(value: CreatePartyAccountingProfileResolutionOutput) -> Self {
        Self {
            default_sales_account_id: value.default_sales_account_id,
            default_purchase_account_id: value.default_purchase_account_id,
            default_receivable_account_id: value.default_receivable_account_id,
            default_payable_account_id: value.default_payable_account_id,
            default_output_tax_account_id: value.default_output_tax_account_id,
            default_input_tax_account_id: value.default_input_tax_account_id,
        }
    }
}

impl IntoServiceInput<CreatePartyAddressInput> for CreatePartyAddressRequestDto {
    fn into_service_input(self) -> CreatePartyAddressInput {
        CreatePartyAddressInput {
            address_type: self.address_type.into_service_input(),
            label: self.label,
            line1: self.line1,
            line2: self.line2,
            city: self.city,
            state_region: self.state_region,
            postal_code: self.postal_code,
            country_iso_code: self.country_iso_code,
            is_default_billing: self.is_default_billing,
            is_default_shipping: self.is_default_shipping,
        }
    }
}

impl IntoServiceInput<CreatePartyContactInput> for CreatePartyContactRequestDto {
    fn into_service_input(self) -> CreatePartyContactInput {
        CreatePartyContactInput {
            name_primary: self.name_primary,
            name_secondary: self.name_secondary,
            designation: self.designation,
            phone: self.phone,
            email: self.email,
            is_primary: self.is_primary,
        }
    }
}

impl PartyTypeDto {
    pub fn into_service_input(self) -> PartyType {
        match self {
            Self::Customer => PartyType::Customer,
            Self::Vendor => PartyType::Vendor,
            Self::Both => PartyType::Both,
        }
    }

    pub fn from_service_output(party_type: PartyType) -> Self {
        match party_type {
            PartyType::Customer => Self::Customer,
            PartyType::Vendor => Self::Vendor,
            PartyType::Both => Self::Both,
        }
    }
}

impl PartyKindDto {
    pub fn into_service_input(self) -> PartyKind {
        match self {
            Self::Person => PartyKind::Person,
            Self::Business => PartyKind::Business,
        }
    }

    pub fn from_service_output(party_kind: PartyKind) -> Self {
        match party_kind {
            PartyKind::Person => Self::Person,
            PartyKind::Business => Self::Business,
        }
    }
}

impl PartyStatusDto {
    pub fn into_service_input(self) -> PartyStatus {
        match self {
            Self::Active => PartyStatus::Active,
            Self::Inactive => PartyStatus::Inactive,
            Self::Deleted => PartyStatus::Deleted,
        }
    }

    pub fn from_service_output(status: PartyStatus) -> Self {
        match status {
            PartyStatus::Active => Self::Active,
            PartyStatus::Inactive => Self::Inactive,
            PartyStatus::Deleted => Self::Deleted,
        }
    }
}

impl PartySourceDto {
    pub fn into_service_input(self) -> PartySource {
        match self {
            Self::Manual => PartySource::Manual,
            Self::PosQuick => PartySource::PosQuick,
            Self::Import => PartySource::Import,
        }
    }

    pub fn from_service_output(source: PartySource) -> Self {
        match source {
            PartySource::Manual => Self::Manual,
            PartySource::PosQuick => Self::PosQuick,
            PartySource::Import => Self::Import,
        }
    }
}

impl PartyAddressTypeDto {
    pub fn into_service_input(self) -> PartyAddressType {
        match self {
            Self::Billing => PartyAddressType::Billing,
            Self::Shipping => PartyAddressType::Shipping,
            Self::Registered => PartyAddressType::Registered,
            Self::Office => PartyAddressType::Office,
            Self::Home => PartyAddressType::Home,
            Self::Other => PartyAddressType::Other,
        }
    }

    pub fn from_service_output(address_type: PartyAddressType) -> Self {
        match address_type {
            PartyAddressType::Billing => Self::Billing,
            PartyAddressType::Shipping => Self::Shipping,
            PartyAddressType::Registered => Self::Registered,
            PartyAddressType::Office => Self::Office,
            PartyAddressType::Home => Self::Home,
            PartyAddressType::Other => Self::Other,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PartyListPageQueryDto {
    #[serde(flatten)]
    pub pagination: PagePaginationQuery,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub party_type: Option<PartyTypeDto>,
    pub status: Option<PartyStatusDto>,
    pub sort: Option<PartySortFieldDto>,
    pub direction: Option<SortDirectionDto>,
}

impl IntoServiceInput<PartyListPageInput> for PartyListPageQueryDto {
    fn into_service_input(self) -> PartyListPageInput {
        PartyListPageInput {
            page: self.pagination.page,
            per_page: self.pagination.per_page,
            name: self.name,
            phone: self.phone,
            party_type: self.party_type.map(PartyTypeDto::into_service_input),
            status: self.status.map(PartyStatusDto::into_service_input),
            sort: self.sort.map(|sort| match sort {
                PartySortFieldDto::CreatedAt => PartySortField::CreatedAt,
                PartySortFieldDto::DisplayName => PartySortField::DisplayName,
                PartySortFieldDto::Code => PartySortField::Code,
            }),
            direction: self.direction.map(|direction| match direction {
                SortDirectionDto::Asc => SortDirection::Asc,
                SortDirectionDto::Desc => SortDirection::Desc,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PartyListItemResponseDto {
    pub public_id: String,
    pub code: String,
    pub party_type: PartyTypeDto,
    pub party_kind: PartyKindDto,
    pub status: PartyStatusDto,
    pub source: PartySourceDto,
    pub display_name: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub legal_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub tax_no: Option<String>,
    pub country_iso_code: Option<String>,
    pub currency_iso_code: Option<String>,
    pub allow_credit: bool,
    pub payment_terms_days: Option<i16>,
    pub credit_limit: Option<Decimal>,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PartyAddressResponseDto {
    pub address_type: PartyAddressTypeDto,
    pub label: Option<String>,
    pub line1: String,
    pub line2: Option<String>,
    pub city: Option<String>,
    pub state_region: Option<String>,
    pub postal_code: Option<String>,
    pub country_iso_code: Option<String>,
    pub is_default_billing: bool,
    pub is_default_shipping: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PartyContactResponseDto {
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub designation: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub is_primary: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PartyAccountingProfileResponseDto {
    pub default_sales_account_public_id: Option<String>,
    pub default_purchase_account_public_id: Option<String>,
    pub default_receivable_account_public_id: Option<String>,
    pub default_payable_account_public_id: Option<String>,
    pub default_output_tax_account_public_id: Option<String>,
    pub default_input_tax_account_public_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PartyDetailResponseDto {
    pub public_id: String,
    pub code: String,
    pub party_type: PartyTypeDto,
    pub party_kind: PartyKindDto,
    pub status: PartyStatusDto,
    pub source: PartySourceDto,
    pub display_name: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub legal_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub tax_no: Option<String>,
    pub tax_treatment: Option<String>,
    pub country_iso_code: Option<String>,
    pub currency_iso_code: Option<String>,
    pub allow_credit: bool,
    pub payment_terms_days: Option<i16>,
    pub credit_limit: Option<Decimal>,
    pub notes: Option<String>,
    pub addresses: Vec<PartyAddressResponseDto>,
    pub contacts: Vec<PartyContactResponseDto>,
    pub accounting_profile: Option<PartyAccountingProfileResponseDto>,
}

impl PartyAddressResponseDto {
    pub fn from_service_output(address: PartyAddressDetail) -> Self {
        Self {
            address_type: PartyAddressTypeDto::from_service_output(address.address_type),
            label: address.label,
            line1: address.line1,
            line2: address.line2,
            city: address.city,
            state_region: address.state_region,
            postal_code: address.postal_code,
            country_iso_code: address.country_iso_code,
            is_default_billing: address.is_default_billing,
            is_default_shipping: address.is_default_shipping,
        }
    }
}

impl PartyContactResponseDto {
    pub fn from_service_output(contact: PartyContactDetail) -> Self {
        Self {
            name_primary: contact.name_primary,
            name_secondary: contact.name_secondary,
            designation: contact.designation,
            phone: contact.phone,
            email: contact.email,
            is_primary: contact.is_primary,
        }
    }
}

impl PartyAccountingProfileResponseDto {
    pub fn from_service_output(accounting_profile: PartyAccountingProfileDetail) -> Self {
        Self {
            default_sales_account_public_id: accounting_profile.default_sales_account_public_id,
            default_purchase_account_public_id: accounting_profile
                .default_purchase_account_public_id,
            default_receivable_account_public_id: accounting_profile
                .default_receivable_account_public_id,
            default_payable_account_public_id: accounting_profile.default_payable_account_public_id,
            default_output_tax_account_public_id: accounting_profile
                .default_output_tax_account_public_id,
            default_input_tax_account_public_id: accounting_profile
                .default_input_tax_account_public_id,
        }
    }
}

impl PartyListItemResponseDto {
    pub fn from_service_output(item: PartyListItem) -> Self {
        Self {
            public_id: item.public_id,
            code: item.code,
            party_type: PartyTypeDto::from_service_output(item.party_type),
            party_kind: PartyKindDto::from_service_output(item.party_kind),
            status: PartyStatusDto::from_service_output(item.status),
            source: PartySourceDto::from_service_output(item.source),
            display_name: item.display_name,
            name_primary: item.name_primary,
            name_secondary: item.name_secondary,
            legal_name: item.legal_name,
            phone: item.phone,
            email: item.email,
            tax_no: item.tax_no,
            country_iso_code: item.country_iso_code,
            currency_iso_code: item.currency_iso_code,
            allow_credit: item.allow_credit,
            payment_terms_days: item.payment_terms_days,
            credit_limit: item.credit_limit,
        }
    }

    pub fn page_from_service_output(result: PageListResult<PartyListItem>) -> PageListResult<Self> {
        result.map_rows(Self::from_service_output)
    }
}

impl PartyDetailResponseDto {
    pub fn from_service_output(detail: PartyDetail) -> Self {
        Self {
            public_id: detail.public_id,
            code: detail.code,
            party_type: PartyTypeDto::from_service_output(detail.party_type),
            party_kind: PartyKindDto::from_service_output(detail.party_kind),
            status: PartyStatusDto::from_service_output(detail.status),
            source: PartySourceDto::from_service_output(detail.source),
            display_name: detail.display_name,
            name_primary: detail.name_primary,
            name_secondary: detail.name_secondary,
            legal_name: detail.legal_name,
            phone: detail.phone,
            email: detail.email,
            tax_no: detail.tax_no,
            tax_treatment: detail.tax_treatment,
            country_iso_code: detail.country_iso_code,
            currency_iso_code: detail.currency_iso_code,
            allow_credit: detail.allow_credit,
            payment_terms_days: detail.payment_terms_days,
            credit_limit: detail.credit_limit,
            notes: detail.notes,
            addresses: detail
                .addresses
                .into_iter()
                .map(PartyAddressResponseDto::from_service_output)
                .collect(),
            contacts: detail
                .contacts
                .into_iter()
                .map(PartyContactResponseDto::from_service_output)
                .collect(),
            accounting_profile: detail
                .accounting_profile
                .map(PartyAccountingProfileResponseDto::from_service_output),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_party_request_deserializes_nested_party_payload() {
        let request: CreatePartyRequestDto = serde_json::from_value(serde_json::json!({
            "code": "CUS-001",
            "partyType": "customer",
            "partyKind": "business",
            "source": "manual",
            "displayName": "Acme Stores",
            "namePrimary": "Acme Stores Pvt Ltd",
            "nameSecondary": null,
            "legalName": "Acme Stores Private Limited",
            "phone": "+919876543210",
            "email": "billing@acme.example",
            "taxNo": "GSTIN123456789",
            "taxTreatment": "registered",
            "countryIsoCode": "IN",
            "currencyIsoCode": "INR",
            "paymentTermsDays": 30,
            "creditLimit": "50000.00",
            "allowCredit": true,
            "notes": null,
            "accountingProfile": {
                "defaultSalesAccountPublicId": "sales_acc",
                "defaultPurchaseAccountPublicId": null,
                "defaultReceivableAccountPublicId": "receivable_acc",
                "defaultPayableAccountPublicId": null,
                "defaultOutputTaxAccountPublicId": "output_tax_acc",
                "defaultInputTaxAccountPublicId": null
            },
            "addresses": [{
                "addressType": "billing",
                "label": "Billing",
                "line1": "12 Market Road",
                "line2": null,
                "city": "Kolkata",
                "stateRegion": "West Bengal",
                "postalCode": "700001",
                "countryIsoCode": "IN",
                "isDefaultBilling": true,
                "isDefaultShipping": false
            }],
            "contacts": [{
                "namePrimary": "Rahul Sen",
                "nameSecondary": null,
                "designation": "Accounts",
                "phone": "+919800000000",
                "email": "rahul@acme.example",
                "isPrimary": true
            }]
        }))
        .unwrap();

        assert_eq!(request.party_type, PartyTypeDto::Customer);
        assert_eq!(request.party_kind, PartyKindDto::Business);
        assert_eq!(request.source, Some(PartySourceDto::Manual));
        assert_eq!(request.display_name, "Acme Stores");
        assert_eq!(
            request
                .accounting_profile
                .as_ref()
                .and_then(|profile| profile.default_sales_account_public_id.as_deref()),
            Some("sales_acc")
        );
        assert_eq!(request.addresses.len(), 1);
        assert_eq!(request.contacts.len(), 1);
    }

    #[test]
    fn pos_quick_source_deserializes_from_camel_case_value() {
        let request: CreatePartyRequestDto = serde_json::from_value(serde_json::json!({
            "code": "POS-001",
            "partyType": "customer",
            "partyKind": "person",
            "source": "posQuick",
            "displayName": "+919876543210",
            "namePrimary": "+919876543210",
            "nameSecondary": null,
            "legalName": null,
            "phone": "+919876543210",
            "email": null,
            "taxNo": null,
            "taxTreatment": null,
            "countryIsoCode": "IN",
            "currencyIsoCode": "INR",
            "paymentTermsDays": null,
            "creditLimit": null,
            "allowCredit": false,
            "notes": null
        }))
        .unwrap();

        assert_eq!(request.source, Some(PartySourceDto::PosQuick));
        assert!(request.addresses.is_empty());
        assert!(request.contacts.is_empty());
    }

    #[test]
    fn party_list_page_query_deserializes_filters_and_sort() {
        let query: PartyListPageQueryDto = serde_json::from_value(serde_json::json!({
            "page": 1,
            "perPage": 20,
            "name": "Acme",
            "phone": "9876",
            "partyType": "both",
            "status": "active",
            "sort": "displayName",
            "direction": "asc"
        }))
        .unwrap();

        assert_eq!(query.pagination.page, 1);
        assert_eq!(query.pagination.per_page, 20);
        assert_eq!(query.party_type, Some(PartyTypeDto::Both));
        assert_eq!(query.status, Some(PartyStatusDto::Active));
        assert_eq!(query.sort, Some(PartySortFieldDto::DisplayName));
        assert_eq!(query.direction, Some(SortDirectionDto::Asc));
    }
}
