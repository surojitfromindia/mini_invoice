use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::common_dto::{IntoServiceInput, PagePaginationQuery};
use crate::db::listing::PageListResult;
use crate::entity::GenericStatus;
use crate::service::unit_service::{
    CreateUnitInput, UnitListItem, UnitListPageInput, UnitSortField,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateUnitRequestDto {
    pub code: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub symbol: Option<String>,
    pub decimal_places: i16,
}

impl IntoServiceInput<CreateUnitInput> for CreateUnitRequestDto {
    fn into_service_input(self) -> CreateUnitInput {
        CreateUnitInput {
            code: self.code,
            name_primary: self.name_primary,
            name_secondary: self.name_secondary,
            symbol: self.symbol,
            decimal_places: self.decimal_places,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum UnitSortFieldDto {
    CreatedAt,
    Code,
    NamePrimary,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum SortDirectionDto {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum GenericStatusDto {
    Active,
    Deleted,
}

impl GenericStatusDto {
    pub fn into_service_input(self) -> GenericStatus {
        match self {
            Self::Active => GenericStatus::Active,
            Self::Deleted => GenericStatus::Deleted,
        }
    }

    pub fn from_service_output(status: GenericStatus) -> Self {
        match status {
            GenericStatus::Active => Self::Active,
            GenericStatus::Deleted => Self::Deleted,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UnitListPageQueryDto {
    #[serde(flatten)]
    pub pagination: PagePaginationQuery,
    pub code: Option<String>,
    pub name: Option<String>,
    pub status: Option<GenericStatusDto>,
    pub is_system_unit: Option<bool>,
    pub sort: Option<UnitSortFieldDto>,
    pub direction: Option<SortDirectionDto>,
}

impl IntoServiceInput<UnitListPageInput> for UnitListPageQueryDto {
    fn into_service_input(self) -> UnitListPageInput {
        UnitListPageInput {
            page: self.pagination.page,
            per_page: self.pagination.per_page,
            code: self.code,
            name: self.name,
            status: self.status.map(GenericStatusDto::into_service_input),
            is_system_unit: self.is_system_unit,
            sort: self.sort.map(|sort| match sort {
                UnitSortFieldDto::CreatedAt => UnitSortField::CreatedAt,
                UnitSortFieldDto::Code => UnitSortField::Code,
                UnitSortFieldDto::NamePrimary => UnitSortField::NamePrimary,
            }),
            direction: self.direction.map(|direction| match direction {
                SortDirectionDto::Asc => crate::service::unit_service::SortDirection::Asc,
                SortDirectionDto::Desc => crate::service::unit_service::SortDirection::Desc,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UnitListItemResponseDto {
    pub public_id: String,
    pub code: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub symbol: Option<String>,
    pub decimal_places: i16,
    pub is_system_unit: bool,
    pub status: GenericStatusDto,
}

impl UnitListItemResponseDto {
    pub fn from_service_output(item: UnitListItem) -> Self {
        Self {
            public_id: item.public_id,
            code: item.code,
            name_primary: item.name_primary,
            name_secondary: item.name_secondary,
            symbol: item.symbol,
            decimal_places: item.decimal_places,
            is_system_unit: item.is_system_unit,
            status: GenericStatusDto::from_service_output(item.status),
        }
    }

    pub fn page_from_service_output(result: PageListResult<UnitListItem>) -> PageListResult<Self> {
        result.map_rows(Self::from_service_output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_unit_request_deserializes_camel_case_keys() {
        let request: CreateUnitRequestDto = serde_json::from_value(serde_json::json!({
            "code": "BAG",
            "namePrimary": "Bag",
            "nameSecondary": null,
            "symbol": "bag",
            "decimalPlaces": 0
        }))
        .unwrap();

        assert_eq!(request.code, "BAG");
        assert_eq!(request.name_primary, "Bag");
        assert_eq!(request.decimal_places, 0);
    }

    #[test]
    fn unit_list_item_response_serializes_camel_case_keys() {
        let response = UnitListItemResponseDto {
            public_id: "unit_123".to_string(),
            code: "BAG".to_string(),
            name_primary: "Bag".to_string(),
            name_secondary: None,
            symbol: Some("bag".to_string()),
            decimal_places: 0,
            is_system_unit: false,
            status: GenericStatusDto::Active,
        };

        let json = serde_json::to_value(response).unwrap();

        assert_eq!(
            json,
            serde_json::json!({
                "publicId": "unit_123",
                "code": "BAG",
                "namePrimary": "Bag",
                "nameSecondary": null,
                "symbol": "bag",
                "decimalPlaces": 0,
                "isSystemUnit": false,
                "status": "active"
            })
        );
    }
}
