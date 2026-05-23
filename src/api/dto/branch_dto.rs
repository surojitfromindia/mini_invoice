use sea_orm::FromQueryResult;
use serde::{Deserialize, Serialize};

use crate::service::branch_service::{
    BranchInclude, BranchListPageInput, BranchSortField, CreateBranchInput, SortDirection,
};

use super::common_dto::PagePaginationQuery;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreateBranchRequestDto {
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub is_primary: Option<bool>,
}

impl From<CreateBranchRequestDto> for CreateBranchInput {
    fn from(value: CreateBranchRequestDto) -> Self {
        Self {
            name_primary: value.name_primary,
            name_secondary: value.name_secondary,
            is_primary: value.is_primary,
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
#[serde(untagged)]
enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
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

#[derive(Debug, Clone, PartialEq, Serialize, FromQueryResult)]
pub struct BranchListItemDto {
    pub public_id: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub is_primary: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_name_primary: Option<String>,
}

impl From<BranchListPageQueryDto> for BranchListPageInput {
    fn from(value: BranchListPageQueryDto) -> Self {
        Self {
            page: value.pagination.page,
            per_page: value.pagination.per_page,
            name: value.name,
            is_primary: value.is_primary,
            sort: value.sort.map(Into::into),
            direction: value.direction.map(Into::into),
            include: value
                .include
                .unwrap_or_default()
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

impl From<BranchSortFieldDto> for BranchSortField {
    fn from(value: BranchSortFieldDto) -> Self {
        match value {
            BranchSortFieldDto::CreatedAt => Self::CreatedAt,
            BranchSortFieldDto::NamePrimary => Self::NamePrimary,
        }
    }
}

impl From<SortDirectionDto> for SortDirection {
    fn from(value: SortDirectionDto) -> Self {
        match value {
            SortDirectionDto::Asc => Self::Asc,
            SortDirectionDto::Desc => Self::Desc,
        }
    }
}

impl From<BranchIncludeDto> for BranchInclude {
    fn from(value: BranchIncludeDto) -> Self {
        match value {
            BranchIncludeDto::Organization => Self::Organization,
        }
    }
}

fn deserialize_optional_one_or_many<'de, D, T>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    let value = Option::<OneOrMany<T>>::deserialize(deserializer)?;
    Ok(value.map(|value| match value {
        OneOrMany::One(item) => vec![item],
        OneOrMany::Many(items) => items,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_branch_request_maps_to_service_input() {
        let input: CreateBranchInput = CreateBranchRequestDto {
            name_primary: "HQ".to_string(),
            name_secondary: Some("Main".to_string()),
            is_primary: Some(true),
        }
        .into();

        assert_eq!(input.name_primary, "HQ");
        assert_eq!(input.name_secondary.as_deref(), Some("Main"));
        assert_eq!(input.is_primary, Some(true));
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
