use std::cmp::Ordering;
use std::collections::HashMap;

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::entity::coa::coa_entity as CoaAccount;
use crate::entity::coa::coa_template_entity as CoaTemplate;
use crate::entity::{GenericStatus, PrimaryId, PublicId};
use crate::errors::app_error::AppError;
use crate::service::service_context::ServiceContext;
use crate::service::tree_graph::TreeGraph;

// COA fetch flow:
// 1. Find the organization's default COA template.
// 2. Load every active account row for that template.
// 3. Rebuild the parent/child structure in memory.
// 4. Return either a flat list or a nested tree.
//
// We keep the database query simple on purpose. The rows already contain the
// parent pointers we need, so Rust can rebuild the tree for us.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoaViewMode {
    Flat,
    Tree,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartOfAccountsTemplate {
    pub id: PrimaryId,
    pub public_id: PublicId,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub description: Option<String>,
    pub country_iso_code: String,
    pub accounting_standard: Option<String>,
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartOfAccountItemFields {
    pub id: PrimaryId,
    pub public_id: PublicId,
    pub code: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub description: Option<String>,
    pub level_no: i16,
    pub is_posting: bool,
    pub is_system_account: bool,
    pub parent_id: Option<PrimaryId>,
    pub parent_public_id: Option<PublicId>,
    pub account_group_id: Option<PrimaryId>,
    pub account_group_public_id: Option<PublicId>,
    pub account_type_id: Option<PrimaryId>,
    pub account_type_public_id: Option<PublicId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartOfAccountFlatItem {
    pub item: ChartOfAccountItemFields,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartOfAccountTreeItem {
    pub item: ChartOfAccountItemFields,
    pub children: Vec<ChartOfAccountTreeItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChartOfAccountsViewResult {
    Flat {
        template: ChartOfAccountsTemplate,
        accounts: Vec<ChartOfAccountFlatItem>,
    },
    Tree {
        template: ChartOfAccountsTemplate,
        accounts: Vec<ChartOfAccountTreeItem>,
    },
}

pub struct CoaService;

impl CoaService {
    pub async fn fetch_default_chart_of_accounts(
        ctx: &ServiceContext,
        view_mode: CoaViewMode,
    ) -> Result<ChartOfAccountsViewResult, AppError> {
        // This is the first safety boundary. Everything we load belongs to the
        // caller's organization.
        let organization_id = ctx.get_organization_id()?;
        let template_model = Self::fetch_default_template(ctx, organization_id).await?;
        let accounts = Self::fetch_template_accounts(ctx, template_model.id).await?;
        // Build the graph once. After this we can render the same data as a
        // flat list or a tree without querying the database again.
        let graph = TreeGraph::try_new(
            accounts,
            |account: &CoaAccount::Model| account.id,
            |account: &CoaAccount::Model| account.parent_account_id,
            "COA account",
        )?;
        let template = Self::template_from_model(template_model);

        match view_mode {
            CoaViewMode::Flat => Ok(ChartOfAccountsViewResult::Flat {
                template,
                accounts: graph.flatten(
                    &|account| Self::flat_item(account, &graph),
                    &compare_account_ids,
                )?,
            }),
            CoaViewMode::Tree => Ok(ChartOfAccountsViewResult::Tree {
                template,
                accounts: graph.build_tree(
                    &|account, children| Self::tree_item(account, children, &graph),
                    &compare_account_ids,
                )?,
            }),
        }
    }

    async fn fetch_default_template(
        ctx: &ServiceContext,
        organization_id: PrimaryId,
    ) -> Result<CoaTemplate::Model, AppError> {
        // A valid organization should have exactly one active default template.
        CoaTemplate::Entity::find()
            .filter(CoaTemplate::Column::OrganizationId.eq(organization_id))
            .filter(CoaTemplate::Column::IsDefault.eq(true))
            .filter(CoaTemplate::Column::Status.eq(GenericStatus::Active))
            .one(&ctx.app_state.primary_read_replica)
            .await?
            .ok_or_else(|| {
                AppError::InternalServer(format!(
                    "Default COA template not found for organization {organization_id}"
                ))
            })
    }

    async fn fetch_template_accounts(
        ctx: &ServiceContext,
        template_id: PrimaryId,
    ) -> Result<Vec<CoaAccount::Model>, AppError> {
        // Load all active rows first. The hierarchy can only be rebuilt safely
        // when parent and child rows are available together in memory.
        CoaAccount::Entity::find()
            .filter(CoaAccount::Column::CoaTemplateId.eq(template_id))
            .filter(CoaAccount::Column::Status.eq(GenericStatus::Active))
            .all(&ctx.app_state.primary_read_replica)
            .await
            .map_err(Into::into)
    }

    fn template_from_model(template: CoaTemplate::Model) -> ChartOfAccountsTemplate {
        ChartOfAccountsTemplate {
            id: template.id,
            public_id: template.public_id,
            name_primary: template.name_primary,
            name_secondary: template.name_secondary,
            description: template.description,
            country_iso_code: template.country_iso_code,
            accounting_standard: template.accounting_standard,
            is_default: template.is_default,
        }
    }

    fn flat_item(
        account: &CoaAccount::Model,
        graph: &TreeGraph<CoaAccount::Model, PrimaryId>,
    ) -> Result<ChartOfAccountFlatItem, AppError> {
        // Convert one database row into the public API shape.
        // We expose public ids instead of internal database ids so the output
        // stays stable for clients.
        Ok(ChartOfAccountFlatItem {
            item: Self::item_fields(account, graph)?,
        })
    }

    fn tree_item(
        account: &CoaAccount::Model,
        children: Vec<ChartOfAccountTreeItem>,
        graph: &TreeGraph<CoaAccount::Model, PrimaryId>,
    ) -> Result<ChartOfAccountTreeItem, AppError> {
        // Tree nodes have the same fields as flat nodes, plus the nested
        // children list.
        Ok(ChartOfAccountTreeItem {
            item: Self::item_fields(account, graph)?,
            children,
        })
    }

    fn item_fields(
        account: &CoaAccount::Model,
        graph: &TreeGraph<CoaAccount::Model, PrimaryId>,
    ) -> Result<ChartOfAccountItemFields, AppError> {
        Ok(ChartOfAccountItemFields {
            id: account.id,
            public_id: account.public_id.clone(),
            code: account.code.clone(),
            name_primary: account.name_primary.clone(),
            name_secondary: account.name_secondary.clone(),
            description: account.description.clone(),
            level_no: account.level_no,
            is_posting: account.is_posting,
            is_system_account: account.is_system_account,
            parent_id: account.parent_account_id,
            parent_public_id: resolve_public_id(
                account.parent_account_id,
                graph,
                "parent account",
            )?,
            account_group_id: account.account_group_id,
            account_group_public_id: resolve_public_id(
                account.account_group_id,
                graph,
                "account group",
            )?,
            account_type_id: account.account_type_id,
            account_type_public_id: resolve_public_id(
                account.account_type_id,
                graph,
                "account type",
            )?,
        })
    }
}

fn resolve_public_id(
    account_id: Option<PrimaryId>,
    graph: &TreeGraph<CoaAccount::Model, PrimaryId>,
    label: &str,
) -> Result<Option<PublicId>, AppError> {
    // Convert an internal database id into the public id we return to clients.
    match account_id {
        Some(account_id) => graph
            .get(account_id)
            .map(|account| Some(account.public_id.clone()))
            .ok_or_else(|| {
                AppError::InternalServer(format!(
                    "Missing {label} `{account_id}` while resolving COA response payload"
                ))
            }),
        None => Ok(None),
    }
}

fn compare_codes(left: &str, right: &str) -> Ordering {
    // Numeric COA codes should sort numerically (`10` before `100`), not as
    // text (`100` before `10`).
    match (left.parse::<u64>(), right.parse::<u64>()) {
        (Ok(left), Ok(right)) => left
            .cmp(&right)
            .then_with(|| left.to_string().cmp(&right.to_string())),
        _ => left.cmp(right),
    }
}

fn compare_account_ids(
    left: PrimaryId,
    right: PrimaryId,
    accounts_by_id: &HashMap<PrimaryId, CoaAccount::Model>,
) -> Ordering {
    // COA codes are numeric strings, so compare by number when possible.
    let left_code = accounts_by_id
        .get(&left)
        .map(|account| account.code.as_str());
    let right_code = accounts_by_id
        .get(&right)
        .map(|account| account.code.as_str());

    match (left_code, right_code) {
        (Some(left_code), Some(right_code)) => compare_codes(left_code, right_code),
        _ => left.cmp(&right),
    }
}
