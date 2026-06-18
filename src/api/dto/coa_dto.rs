use crate::entity::PrimaryId;
use crate::service::coa_service::{
    ChartOfAccountFlatItem, ChartOfAccountItemFields, ChartOfAccountTreeItem,
    ChartOfAccountsTemplate, ChartOfAccountsViewResult, CoaViewMode, CreateChartOfAccountInput,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::common_dto::IntoServiceInput;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ChartOfAccountsViewModeDto {
    Flat,
    Tree,
}

impl Default for ChartOfAccountsViewModeDto {
    fn default() -> Self {
        Self::Tree
    }
}

impl IntoServiceInput<CoaViewMode> for ChartOfAccountsViewModeDto {
    fn into_service_input(self) -> CoaViewMode {
        match self {
            Self::Flat => CoaViewMode::Flat,
            Self::Tree => CoaViewMode::Tree,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChartOfAccountsQueryDto {
    pub view: Option<ChartOfAccountsViewModeDto>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateChartOfAccountRequestDto {
    pub code: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub description: Option<String>,
    pub parent_account_public_id: String,
}

pub struct CreateChartOfAccountResolutionInput {
    pub code: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub description: Option<String>,
    pub parent_account_public_id: String,
}

impl CreateChartOfAccountRequestDto {
    pub fn into_resolution_input(self) -> CreateChartOfAccountResolutionInput {
        CreateChartOfAccountResolutionInput {
            code: self.code,
            name_primary: self.name_primary,
            name_secondary: self.name_secondary,
            description: self.description,
            parent_account_public_id: self.parent_account_public_id,
        }
    }
}

impl CreateChartOfAccountResolutionInput {
    pub fn into_service_input(self, parent_account_id: PrimaryId) -> CreateChartOfAccountInput {
        CreateChartOfAccountInput {
            code: self.code,
            name_primary: self.name_primary,
            name_secondary: self.name_secondary,
            description: self.description,
            parent_account_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChartOfAccountsTemplateDto {
    pub public_id: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub description: Option<String>,
    pub country_iso_code: String,
    pub accounting_standard: Option<String>,
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChartOfAccountItemDto {
    pub id: PrimaryId,
    pub public_id: String,
    pub code: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub description: Option<String>,
    pub level_no: i16,
    pub is_posting: bool,
    pub is_system_account: bool,
    pub parent_id: Option<PrimaryId>,
    pub parent_public_id: Option<String>,
    pub account_group_id: Option<PrimaryId>,
    pub account_group_public_id: Option<String>,
    pub account_type_id: Option<PrimaryId>,
    pub account_type_public_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChartOfAccountFlatItemDto {
    #[serde(flatten)]
    pub item: ChartOfAccountItemDto,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChartOfAccountTreeItemDto {
    #[serde(flatten)]
    pub item: ChartOfAccountItemDto,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<ChartOfAccountTreeItemDto>,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(tag = "view", rename_all = "camelCase")]
pub enum ChartOfAccountsResponseDto {
    Flat {
        template: ChartOfAccountsTemplateDto,
        accounts: Vec<ChartOfAccountFlatItemDto>,
    },
    Tree {
        template: ChartOfAccountsTemplateDto,
        accounts: Vec<ChartOfAccountTreeItemDto>,
    },
}

impl ChartOfAccountsTemplateDto {
    fn from_service_output(template: ChartOfAccountsTemplate) -> Self {
        Self {
            public_id: template.public_id,
            name_primary: template.name_primary,
            name_secondary: template.name_secondary,
            description: template.description,
            country_iso_code: template.country_iso_code,
            accounting_standard: template.accounting_standard,
            is_default: template.is_default,
        }
    }
}

impl ChartOfAccountFlatItemDto {
    fn from_service_output(account: ChartOfAccountFlatItem) -> Self {
        Self {
            item: ChartOfAccountItemDto::from_service_output(account.item),
        }
    }
}

impl ChartOfAccountTreeItemDto {
    fn from_service_output(account: ChartOfAccountTreeItem) -> Self {
        Self {
            item: ChartOfAccountItemDto::from_service_output(account.item),
            children: account
                .children
                .into_iter()
                .map(Self::from_service_output)
                .collect(),
        }
    }
}

impl ChartOfAccountItemDto {
    pub fn from_service_output(account: ChartOfAccountItemFields) -> Self {
        Self {
            id: account.id,
            public_id: account.public_id,
            code: account.code,
            name_primary: account.name_primary,
            name_secondary: account.name_secondary,
            description: account.description,
            level_no: account.level_no,
            is_posting: account.is_posting,
            is_system_account: account.is_system_account,
            parent_id: account.parent_id,
            parent_public_id: account.parent_public_id,
            account_group_id: account.account_group_id,
            account_group_public_id: account.account_group_public_id,
            account_type_id: account.account_type_id,
            account_type_public_id: account.account_type_public_id,
        }
    }
}

impl ChartOfAccountsResponseDto {
    pub fn from_service_output(output: ChartOfAccountsViewResult) -> Self {
        match output {
            ChartOfAccountsViewResult::Flat { template, accounts } => Self::Flat {
                template: ChartOfAccountsTemplateDto::from_service_output(template),
                accounts: accounts
                    .into_iter()
                    .map(ChartOfAccountFlatItemDto::from_service_output)
                    .collect(),
            },
            ChartOfAccountsViewResult::Tree { template, accounts } => Self::Tree {
                template: ChartOfAccountsTemplateDto::from_service_output(template),
                accounts: accounts
                    .into_iter()
                    .map(ChartOfAccountTreeItemDto::from_service_output)
                    .collect(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chart_of_accounts_query_defaults_to_tree() {
        let query: ChartOfAccountsQueryDto = serde_json::from_str("{}").unwrap();

        assert_eq!(query.view, None);
    }

    #[test]
    fn chart_of_accounts_query_deserializes_flat_view() {
        let query: ChartOfAccountsQueryDto =
            serde_json::from_value(serde_json::json!({ "view": "flat" })).unwrap();

        assert_eq!(query.view, Some(ChartOfAccountsViewModeDto::Flat));
    }

    #[test]
    fn create_chart_of_account_request_deserializes_camel_case_keys() {
        let request: CreateChartOfAccountRequestDto = serde_json::from_value(serde_json::json!({
            "code": "1019",
            "namePrimary": "Operating Bank",
            "nameSecondary": null,
            "description": "Main checking account",
            "parentAccountPublicId": "coa_parent_1"
        }))
        .unwrap();

        assert_eq!(request.code, "1019");
        assert_eq!(request.name_primary, "Operating Bank");
        assert_eq!(request.parent_account_public_id, "coa_parent_1");
    }

    #[test]
    fn chart_of_account_flat_item_keeps_internal_and_public_ids() {
        let dto = ChartOfAccountFlatItemDto::from_service_output(ChartOfAccountFlatItem {
            item: ChartOfAccountItemFields {
                id: 42,
                public_id: "coa_pub_42".to_string(),
                code: "1000".to_string(),
                name_primary: "Cash".to_string(),
                name_secondary: None,
                description: None,
                level_no: 1,
                is_posting: true,
                is_system_account: false,
                parent_id: None,
                parent_public_id: None,
                account_group_id: None,
                account_group_public_id: None,
                account_type_id: None,
                account_type_public_id: None,
            },
        });

        let value = serde_json::to_value(dto).unwrap();

        assert_eq!(value["id"], 42);
        assert_eq!(value["publicId"], "coa_pub_42");
    }

    #[test]
    fn chart_of_account_tree_item_keeps_internal_and_public_ids() {
        let dto = ChartOfAccountTreeItemDto::from_service_output(ChartOfAccountTreeItem {
            item: ChartOfAccountItemFields {
                id: 7,
                public_id: "coa_pub_7".to_string(),
                code: "2000".to_string(),
                name_primary: "Assets".to_string(),
                name_secondary: None,
                description: None,
                level_no: 1,
                is_posting: false,
                is_system_account: true,
                parent_id: None,
                parent_public_id: None,
                account_group_id: None,
                account_group_public_id: None,
                account_type_id: None,
                account_type_public_id: None,
            },
            children: vec![],
        });

        let value = serde_json::to_value(dto).unwrap();

        assert_eq!(value["id"], 7);
        assert_eq!(value["publicId"], "coa_pub_7");
    }
}
