use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

use super::common_dto::{PagePaginationQuery, deserialize_optional_one_or_many};
use crate::db::listing::PageListResult;
use crate::service::branch_service::{
    BranchInclude, BranchListItem, BranchListPageInput, BranchSortField, CreateBranchInput,
    SortDirection,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateBranchRequestDto {
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub is_primary: Option<bool>,
}

impl CreateBranchRequestDto {
    pub fn into_service_input(self) -> CreateBranchInput {
        CreateBranchInput {
            name_primary: self.name_primary,
            name_secondary: self.name_secondary,
            is_primary: self.is_primary,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BranchSortFieldDto {
    CreatedAt,
    NamePrimary,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortDirectionDto {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BranchIncludeDto {
    Organization,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct BranchListPageQueryDto {
    #[serde(flatten)]
    pub pagination: PagePaginationQuery,
    pub name: Option<String>,
    pub is_primary: Option<bool>,
    pub sort: Option<BranchSortFieldDto>,
    pub direction: Option<SortDirectionDto>,
    #[serde(default, deserialize_with = "deserialize_optional_one_or_many")]
    pub include: Option<Vec<BranchIncludeDto>>,
}

impl BranchListPageQueryDto {
    pub fn into_service_input(self) -> BranchListPageInput {
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
            include: self
                .include
                .unwrap_or_default()
                .into_iter()
                .map(|include| match include {
                    BranchIncludeDto::Organization => BranchInclude::Organization,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, FromQueryResult)]
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
    fn branch_list_page_query_accepts_single_include_value() {
        let query: BranchListPageQueryDto = serde_json::from_value(serde_json::json!({
            "page": 1,
            "per_page": 20,
            "include": "organization"
        }))
        .unwrap();

        assert_eq!(query.include, Some(vec![BranchIncludeDto::Organization]));
    }

    #[test]
    fn branch_list_page_query_accepts_include_array() {
        let query: BranchListPageQueryDto = serde_json::from_value(serde_json::json!({
            "page": 1,
            "per_page": 20,
            "include": ["organization"]
        }))
        .unwrap();

        assert_eq!(query.include, Some(vec![BranchIncludeDto::Organization]));
    }
}
