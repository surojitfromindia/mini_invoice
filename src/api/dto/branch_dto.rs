use schemars::JsonSchema;
use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

use super::common_dto::{IntoServiceInput, PagePaginationQuery};
use crate::db::listing::PageListResult;
use crate::service::branch_service::{
    BranchListItem, BranchListPageInput, BranchSortField, CreateBranchInput, SortDirection,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateBranchRequestDto {
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub is_primary: Option<bool>,
}

impl IntoServiceInput<CreateBranchInput> for CreateBranchRequestDto {
    fn into_service_input(self) -> CreateBranchInput {
        CreateBranchInput {
            name_primary: self.name_primary,
            name_secondary: self.name_secondary,
            is_primary: self.is_primary,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum BranchSortFieldDto {
    CreatedAt,
    NamePrimary,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum SortDirectionDto {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BranchListPageQueryDto {
    #[serde(flatten)]
    pub pagination: PagePaginationQuery,
    pub name: Option<String>,
    pub is_primary: Option<bool>,
    pub sort: Option<BranchSortFieldDto>,
    pub direction: Option<SortDirectionDto>,
}

impl IntoServiceInput<BranchListPageInput> for BranchListPageQueryDto {
    fn into_service_input(self) -> BranchListPageInput {
        BranchListPageInput {
            page: self.pagination.page,
            per_page: self.pagination.per_page,
            name: self.name,
            is_primary: self.is_primary,
            sort: self.sort.map(|sort| match sort {
                BranchSortFieldDto::CreatedAt => BranchSortField::CreatedAt,
                BranchSortFieldDto::NamePrimary => BranchSortField::NamePrimary,
            }),
            direction: self.direction.map(|direction| match direction {
                SortDirectionDto::Asc => SortDirection::Asc,
                SortDirectionDto::Desc => SortDirection::Desc,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, FromQueryResult, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BranchListItemResponseDto {
    pub public_id: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub is_primary: bool,
}

impl BranchListItemResponseDto {
    pub fn from_service_output(item: BranchListItem) -> Self {
        Self {
            public_id: item.public_id,
            name_primary: item.name_primary,
            name_secondary: item.name_secondary,
            is_primary: item.is_primary,
        }
    }

    pub fn page_from_service_output(
        result: PageListResult<BranchListItem>,
    ) -> PageListResult<Self> {
        result.map_rows(Self::from_service_output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn branch_list_item_response_serializes_camel_case_keys() {
        let response = BranchListItemResponseDto {
            public_id: "br_123".to_string(),
            name_primary: "HQ".to_string(),
            name_secondary: Some("Main".to_string()),
            is_primary: true,
        };

        let json = serde_json::to_value(response).unwrap();

        assert_eq!(
            json,
            serde_json::json!({
                "publicId": "br_123",
                "namePrimary": "HQ",
                "nameSecondary": "Main",
                "isPrimary": true
            })
        );
    }

    #[test]
    fn branch_list_page_query_deserializes_filters_and_sort() {
        let query: BranchListPageQueryDto = serde_json::from_value(serde_json::json!({
            "page": 1,
            "perPage": 20,
            "name": "HQ",
            "isPrimary": true,
            "sort": "namePrimary",
            "direction": "asc"
        }))
        .unwrap();

        assert_eq!(query.pagination.page, 1);
        assert_eq!(query.pagination.per_page, 20);
        assert_eq!(query.name.as_deref(), Some("HQ"));
        assert_eq!(query.is_primary, Some(true));
        assert_eq!(query.sort, Some(BranchSortFieldDto::NamePrimary));
        assert_eq!(query.direction, Some(SortDirectionDto::Asc));
    }
}
