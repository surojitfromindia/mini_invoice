use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::common_dto::PagePaginationQuery;
use crate::db::listing::PageListResult;
use crate::entity::auto_number::auto_number_series_entity::{
    AutoNumberResetPolicy, AutoNumberStatus,
};
use crate::service::auto_number_service::{
    AutoNumberSeriesDetail, AutoNumberSeriesListItem, AutoNumberSeriesListPageInput,
    AutoNumberSeriesSortField, CreateAutoNumberSeriesInput, SortDirection,
    UpdateAutoNumberSeriesInput,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum AutoNumberResetPolicyDto {
    Never,
    Monthly,
    CalendarYear,
    FiscalYear,
}

impl AutoNumberResetPolicyDto {
    pub fn into_service_input(self) -> AutoNumberResetPolicy {
        match self {
            Self::Never => AutoNumberResetPolicy::Never,
            Self::Monthly => AutoNumberResetPolicy::Monthly,
            Self::CalendarYear => AutoNumberResetPolicy::CalendarYear,
            Self::FiscalYear => AutoNumberResetPolicy::FiscalYear,
        }
    }

    pub fn from_service_output(reset_policy: AutoNumberResetPolicy) -> Self {
        match reset_policy {
            AutoNumberResetPolicy::Never => Self::Never,
            AutoNumberResetPolicy::Monthly => Self::Monthly,
            AutoNumberResetPolicy::CalendarYear => Self::CalendarYear,
            AutoNumberResetPolicy::FiscalYear => Self::FiscalYear,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum AutoNumberStatusDto {
    Active,
    Inactive,
    Deleted,
}

impl AutoNumberStatusDto {
    pub fn into_service_input(self) -> AutoNumberStatus {
        match self {
            Self::Active => AutoNumberStatus::Active,
            Self::Inactive => AutoNumberStatus::Inactive,
            Self::Deleted => AutoNumberStatus::Deleted,
        }
    }

    pub fn from_service_output(status: AutoNumberStatus) -> Self {
        match status {
            AutoNumberStatus::Active => Self::Active,
            AutoNumberStatus::Inactive => Self::Inactive,
            AutoNumberStatus::Deleted => Self::Deleted,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateAutoNumberSeriesRequestDto {
    pub branch_public_id: String,
    pub series_key: String,
    pub prefix_template: String,
    pub suffix_template: Option<String>,
    pub padding_width: i16,
    pub start_number: i64,
    pub increment_by: i16,
    pub reset_policy: AutoNumberResetPolicyDto,
    pub status: Option<AutoNumberStatusDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAutoNumberSeriesRequestDto {
    pub branch_public_id: Option<String>,
    pub series_key: Option<String>,
    pub prefix_template: Option<String>,
    pub suffix_template: Option<String>,
    pub padding_width: Option<i16>,
    pub start_number: Option<i64>,
    pub increment_by: Option<i16>,
    pub reset_policy: Option<AutoNumberResetPolicyDto>,
    pub status: Option<AutoNumberStatusDto>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum AutoNumberSeriesSortFieldDto {
    CreatedAt,
    SeriesKey,
    Branch,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum SortDirectionDto {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AutoNumberSeriesListPageQueryDto {
    #[serde(flatten)]
    pub pagination: PagePaginationQuery,
    pub branch_public_id: Option<String>,
    pub series_key: Option<String>,
    pub status: Option<AutoNumberStatusDto>,
    pub sort: Option<AutoNumberSeriesSortFieldDto>,
    pub direction: Option<SortDirectionDto>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoNumberSeriesListPageResolutionInput {
    pub page: u64,
    pub per_page: u64,
    pub branch_public_id: Option<String>,
    pub series_key: Option<String>,
    pub status: Option<AutoNumberStatusDto>,
    pub sort: Option<AutoNumberSeriesSortFieldDto>,
    pub direction: Option<SortDirectionDto>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateAutoNumberSeriesResolutionInput {
    pub branch_public_id: String,
    pub series_key: String,
    pub prefix_template: String,
    pub suffix_template: Option<String>,
    pub padding_width: i16,
    pub start_number: i64,
    pub increment_by: i16,
    pub reset_policy: AutoNumberResetPolicyDto,
    pub status: Option<AutoNumberStatusDto>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateAutoNumberSeriesResolutionInput {
    pub branch_public_id: Option<String>,
    pub series_key: Option<String>,
    pub prefix_template: Option<String>,
    pub suffix_template: Option<String>,
    pub padding_width: Option<i16>,
    pub start_number: Option<i64>,
    pub increment_by: Option<i16>,
    pub reset_policy: Option<AutoNumberResetPolicyDto>,
    pub status: Option<AutoNumberStatusDto>,
}

impl CreateAutoNumberSeriesRequestDto {
    pub fn into_resolution_input(self) -> CreateAutoNumberSeriesResolutionInput {
        CreateAutoNumberSeriesResolutionInput {
            branch_public_id: self.branch_public_id,
            series_key: self.series_key,
            prefix_template: self.prefix_template,
            suffix_template: self.suffix_template,
            padding_width: self.padding_width,
            start_number: self.start_number,
            increment_by: self.increment_by,
            reset_policy: self.reset_policy,
            status: self.status,
        }
    }
}

impl UpdateAutoNumberSeriesRequestDto {
    pub fn into_resolution_input(self) -> UpdateAutoNumberSeriesResolutionInput {
        UpdateAutoNumberSeriesResolutionInput {
            branch_public_id: self.branch_public_id,
            series_key: self.series_key,
            prefix_template: self.prefix_template,
            suffix_template: self.suffix_template,
            padding_width: self.padding_width,
            start_number: self.start_number,
            increment_by: self.increment_by,
            reset_policy: self.reset_policy,
            status: self.status,
        }
    }
}

impl AutoNumberSeriesListPageQueryDto {
    pub fn into_resolution_input(self) -> AutoNumberSeriesListPageResolutionInput {
        AutoNumberSeriesListPageResolutionInput {
            page: self.pagination.page,
            per_page: self.pagination.per_page,
            branch_public_id: self.branch_public_id,
            series_key: self.series_key,
            status: self.status,
            sort: self.sort,
            direction: self.direction,
        }
    }
}

impl AutoNumberSeriesListPageResolutionInput {
    pub fn into_service_input(
        self,
        branch_id: Option<crate::entity::PrimaryId>,
    ) -> AutoNumberSeriesListPageInput {
        AutoNumberSeriesListPageInput {
            page: self.page,
            per_page: self.per_page,
            branch_id,
            series_key: self.series_key,
            status: self.status.map(AutoNumberStatusDto::into_service_input),
            sort: self.sort.map(|sort| match sort {
                AutoNumberSeriesSortFieldDto::CreatedAt => AutoNumberSeriesSortField::CreatedAt,
                AutoNumberSeriesSortFieldDto::SeriesKey => AutoNumberSeriesSortField::SeriesKey,
                AutoNumberSeriesSortFieldDto::Branch => AutoNumberSeriesSortField::Branch,
            }),
            direction: self.direction.map(|direction| match direction {
                SortDirectionDto::Asc => SortDirection::Asc,
                SortDirectionDto::Desc => SortDirection::Desc,
            }),
        }
    }
}

impl CreateAutoNumberSeriesResolutionInput {
    pub fn into_service_input(
        self,
        branch_id: crate::entity::PrimaryId,
    ) -> CreateAutoNumberSeriesInput {
        CreateAutoNumberSeriesInput {
            branch_id,
            series_key: self.series_key,
            prefix_template: self.prefix_template,
            suffix_template: self.suffix_template,
            padding_width: self.padding_width,
            start_number: self.start_number,
            increment_by: self.increment_by,
            reset_policy: self.reset_policy.into_service_input(),
            status: self.status.map(AutoNumberStatusDto::into_service_input),
        }
    }
}

impl UpdateAutoNumberSeriesResolutionInput {
    pub fn into_service_input(
        self,
        branch_id: Option<crate::entity::PrimaryId>,
    ) -> UpdateAutoNumberSeriesInput {
        UpdateAutoNumberSeriesInput {
            branch_id,
            series_key: self.series_key,
            prefix_template: self.prefix_template,
            suffix_template: self.suffix_template,
            padding_width: self.padding_width,
            start_number: self.start_number,
            increment_by: self.increment_by,
            reset_policy: self
                .reset_policy
                .map(AutoNumberResetPolicyDto::into_service_input),
            status: self.status.map(AutoNumberStatusDto::into_service_input),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AutoNumberSeriesResponseDto {
    pub public_id: String,
    pub branch_public_id: String,
    pub series_key: String,
    pub prefix_template: String,
    pub suffix_template: Option<String>,
    pub padding_width: i16,
    pub start_number: i64,
    pub increment_by: i16,
    pub reset_policy: AutoNumberResetPolicyDto,
    pub status: AutoNumberStatusDto,
}

impl AutoNumberSeriesResponseDto {
    pub fn from_detail(detail: AutoNumberSeriesDetail) -> Self {
        Self {
            public_id: detail.public_id,
            branch_public_id: detail.branch_public_id,
            series_key: detail.series_key,
            prefix_template: detail.prefix_template,
            suffix_template: detail.suffix_template,
            padding_width: detail.padding_width,
            start_number: detail.start_number,
            increment_by: detail.increment_by,
            reset_policy: AutoNumberResetPolicyDto::from_service_output(detail.reset_policy),
            status: AutoNumberStatusDto::from_service_output(detail.status),
        }
    }

    pub fn from_list_item(item: AutoNumberSeriesListItem) -> Self {
        Self {
            public_id: item.public_id,
            branch_public_id: item.branch_public_id,
            series_key: item.series_key,
            prefix_template: item.prefix_template,
            suffix_template: item.suffix_template,
            padding_width: item.padding_width,
            start_number: item.start_number,
            increment_by: item.increment_by,
            reset_policy: AutoNumberResetPolicyDto::from_service_output(item.reset_policy),
            status: AutoNumberStatusDto::from_service_output(item.status),
        }
    }

    pub fn page_from_service_output(
        result: PageListResult<AutoNumberSeriesListItem>,
    ) -> PageListResult<Self> {
        result.map_rows(Self::from_list_item)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_auto_number_request_deserializes_camel_case_keys() {
        let request: CreateAutoNumberSeriesRequestDto = serde_json::from_value(serde_json::json!({
            "branchPublicId": "branch_1",
            "seriesKey": "customer",
            "prefixTemplate": "CUS-",
            "suffixTemplate": null,
            "paddingWidth": 4,
            "startNumber": 1,
            "incrementBy": 1,
            "resetPolicy": "never",
            "status": "active"
        }))
        .unwrap();

        assert_eq!(request.branch_public_id, "branch_1");
        assert_eq!(request.series_key, "customer");
        assert_eq!(request.reset_policy, AutoNumberResetPolicyDto::Never);
    }
}
