use crate::service::coa_service::{
    ChartOfAccountFlatItem, ChartOfAccountTreeItem, ChartOfAccountsTemplate,
    ChartOfAccountsViewResult, CoaViewMode,
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
pub struct ChartOfAccountFlatItemDto {
    pub public_id: String,
    pub code: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub description: Option<String>,
    pub level_no: i16,
    pub is_posting: bool,
    pub is_system_account: bool,
    pub parent_public_id: Option<String>,
    pub account_group_public_id: Option<String>,
    pub account_type_public_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChartOfAccountTreeItemDto {
    pub public_id: String,
    pub code: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub description: Option<String>,
    pub level_no: i16,
    pub is_posting: bool,
    pub is_system_account: bool,
    pub parent_public_id: Option<String>,
    pub account_group_public_id: Option<String>,
    pub account_type_public_id: Option<String>,
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
            public_id: account.public_id,
            code: account.code,
            name_primary: account.name_primary,
            name_secondary: account.name_secondary,
            description: account.description,
            level_no: account.level_no,
            is_posting: account.is_posting,
            is_system_account: account.is_system_account,
            parent_public_id: account.parent_public_id,
            account_group_public_id: account.account_group_public_id,
            account_type_public_id: account.account_type_public_id,
        }
    }
}

impl ChartOfAccountTreeItemDto {
    fn from_service_output(account: ChartOfAccountTreeItem) -> Self {
        Self {
            public_id: account.public_id,
            code: account.code,
            name_primary: account.name_primary,
            name_secondary: account.name_secondary,
            description: account.description,
            level_no: account.level_no,
            is_posting: account.is_posting,
            is_system_account: account.is_system_account,
            parent_public_id: account.parent_public_id,
            account_group_public_id: account.account_group_public_id,
            account_type_public_id: account.account_type_public_id,
            children: account
                .children
                .into_iter()
                .map(Self::from_service_output)
                .collect(),
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
}
