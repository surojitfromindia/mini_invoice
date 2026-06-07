use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::entity::coa::coa_entity as CoaAccount;
use crate::entity::coa::coa_template_entity as CoaTemplate;
use crate::entity::{GenericStatus, PrimaryId, PublicId};
use crate::errors::app_error::AppError;
use crate::service::service_context::ServiceContext;

// COA fetch flow:
// 1. Resolve the organization's default template.
// 2. Load all active accounts for that template in one query.
// 3. Rebuild the hierarchy in memory using `parent_account_id`.
// 4. Return either a flat pre-order list or a nested tree.
//
// We intentionally keep the database model simple and do not rely on ORM
// relationship loading here. That makes the API flexible while the COA tree
// structure is still driven by the stored parent pointers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoaViewMode {
    Flat,
    Tree,
}

impl Default for CoaViewMode {
    fn default() -> Self {
        Self::Tree
    }
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
pub struct ChartOfAccountFlatItem {
    pub public_id: PublicId,
    pub code: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub description: Option<String>,
    pub level_no: i16,
    pub is_posting: bool,
    pub is_system_account: bool,
    pub parent_public_id: Option<PublicId>,
    pub account_group_public_id: Option<PublicId>,
    pub account_type_public_id: Option<PublicId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChartOfAccountTreeItem {
    pub public_id: PublicId,
    pub code: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub description: Option<String>,
    pub level_no: i16,
    pub is_posting: bool,
    pub is_system_account: bool,
    pub parent_public_id: Option<PublicId>,
    pub account_group_public_id: Option<PublicId>,
    pub account_type_public_id: Option<PublicId>,
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
        // The organization context is the first boundary. Everything we load is
        // scoped to the caller's organization and its default template.
        let organization_id = ctx.get_organization_id()?;
        let template_model = Self::fetch_default_template(ctx, organization_id).await?;
        let accounts = Self::fetch_template_accounts(ctx, template_model.id).await?;
        // Build the graph once, then reuse it to render either flat or tree view.
        let graph = AccountGraph::try_new(accounts)?;
        let template = Self::template_from_model(template_model);

        match view_mode {
            CoaViewMode::Flat => Ok(ChartOfAccountsViewResult::Flat {
                template,
                accounts: graph.flatten()?,
            }),
            CoaViewMode::Tree => Ok(ChartOfAccountsViewResult::Tree {
                template,
                accounts: graph.build_tree()?,
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
}

struct AccountGraph {
    accounts_by_id: HashMap<PrimaryId, CoaAccount::Model>,
    children_by_parent_id: HashMap<PrimaryId, Vec<PrimaryId>>,
    root_ids: Vec<PrimaryId>,
}

impl AccountGraph {
    fn try_new(accounts: Vec<CoaAccount::Model>) -> Result<Self, AppError> {
        let mut accounts_by_id = HashMap::with_capacity(accounts.len());

        for account in accounts {
            accounts_by_id.insert(account.id, account);
        }

        let mut children_by_parent_id: HashMap<PrimaryId, Vec<PrimaryId>> = HashMap::new();
        let mut root_ids = Vec::new();

        // Convert the raw rows into an adjacency list and validate that every
        // stored pointer references a row that exists in the same template.
        for account in accounts_by_id.values() {
            if let Some(parent_id) = account.parent_account_id {
                ensure_account_exists(parent_id, &accounts_by_id, "parent account", account.id)?;
                children_by_parent_id
                    .entry(parent_id)
                    .or_default()
                    .push(account.id);
            } else {
                root_ids.push(account.id);
            }

            if let Some(account_group_id) = account.account_group_id {
                ensure_account_exists(
                    account_group_id,
                    &accounts_by_id,
                    "account group",
                    account.id,
                )?;
            }

            if let Some(account_type_id) = account.account_type_id {
                ensure_account_exists(
                    account_type_id,
                    &accounts_by_id,
                    "account type",
                    account.id,
                )?;
            }
        }

        let mut graph = Self {
            accounts_by_id,
            children_by_parent_id,
            root_ids,
        };
        graph.sort_root_ids();

        Ok(graph)
    }

    fn flatten(&self) -> Result<Vec<ChartOfAccountFlatItem>, AppError> {
        let mut flat_items = Vec::with_capacity(self.accounts_by_id.len());
        let mut visited = HashSet::with_capacity(self.accounts_by_id.len());

        // Flat output is still tree-ordered: parents appear before descendants.
        for root_id in &self.root_ids {
            self.flatten_from(*root_id, &mut visited, &mut flat_items, &mut Vec::new())?;
        }

        self.ensure_all_nodes_visited(&visited)?;
        Ok(flat_items)
    }

    fn build_tree(&self) -> Result<Vec<ChartOfAccountTreeItem>, AppError> {
        let mut visited = HashSet::with_capacity(self.accounts_by_id.len());
        let mut tree = Vec::with_capacity(self.root_ids.len());

        // Tree output reuses the same traversal, but nests each child list under
        // its parent node.
        for root_id in &self.root_ids {
            tree.push(self.build_tree_from(*root_id, &mut visited, &mut Vec::new())?);
        }

        self.ensure_all_nodes_visited(&visited)?;
        Ok(tree)
    }

    fn flatten_from(
        &self,
        account_id: PrimaryId,
        visited: &mut HashSet<PrimaryId>,
        flat_items: &mut Vec<ChartOfAccountFlatItem>,
        stack: &mut Vec<PrimaryId>,
    ) -> Result<(), AppError> {
        // `visited` prevents duplicate emission. `stack` would be used to flag
        // cycles if the tree ever becomes recursive in a bad way.
        if !visited.insert(account_id) {
            return Err(AppError::InternalServer(format!(
                "Duplicate COA node `{account_id}` encountered while flattening tree"
            )));
        }

        stack.push(account_id);
        let account = self.account(account_id)?;
        flat_items.push(self.to_flat_item(account)?);

        for child_id in self.sorted_child_ids(account_id)? {
            self.flatten_from(child_id, visited, flat_items, stack)?;
        }

        stack.pop();
        Ok(())
    }

    fn build_tree_from(
        &self,
        account_id: PrimaryId,
        visited: &mut HashSet<PrimaryId>,
        stack: &mut Vec<PrimaryId>,
    ) -> Result<ChartOfAccountTreeItem, AppError> {
        // A node may not appear twice on the current recursion stack. If it
        // does, the stored parent pointers contain a cycle.
        if stack.contains(&account_id) {
            return Err(AppError::InternalServer(format!(
                "Cycle detected in COA tree at account `{account_id}`"
            )));
        }

        if !visited.insert(account_id) {
            return Err(AppError::InternalServer(format!(
                "Duplicate COA node `{account_id}` encountered while building tree"
            )));
        }

        stack.push(account_id);
        let account = self.account(account_id)?;
        let children = self
            .sorted_child_ids(account_id)?
            .into_iter()
            .map(|child_id| self.build_tree_from(child_id, visited, stack))
            .collect::<Result<Vec<_>, _>>()?;
        stack.pop();

        Ok(self.to_tree_item(account, children)?)
    }

    fn ensure_all_nodes_visited(&self, visited: &HashSet<PrimaryId>) -> Result<(), AppError> {
        // If some rows were never reached from any root, the tree is malformed
        // or disconnected and we should fail loudly.
        if visited.len() == self.accounts_by_id.len() {
            return Ok(());
        }

        let missing_ids: Vec<String> = self
            .accounts_by_id
            .keys()
            .filter(|account_id| !visited.contains(account_id))
            .map(|account_id| account_id.to_string())
            .collect();

        Err(AppError::InternalServer(format!(
            "COA tree is disconnected or cyclic; unreachable account ids: {}",
            missing_ids.join(", ")
        )))
    }

    fn sort_root_ids(&mut self) {
        let accounts_by_id = &self.accounts_by_id;
        self.root_ids
            .sort_by(|left, right| compare_account_ids(*left, *right, accounts_by_id));
    }

    fn sorted_child_ids(&self, account_id: PrimaryId) -> Result<Vec<PrimaryId>, AppError> {
        let mut child_ids = self
            .children_by_parent_id
            .get(&account_id)
            .cloned()
            .unwrap_or_default();
        child_ids.sort_by(|left, right| compare_account_ids(*left, *right, &self.accounts_by_id));
        Ok(child_ids)
    }

    fn account(&self, account_id: PrimaryId) -> Result<&CoaAccount::Model, AppError> {
        self.accounts_by_id.get(&account_id).ok_or_else(|| {
            AppError::InternalServer(format!(
                "COA account `{account_id}` is missing from the current template graph"
            ))
        })
    }

    fn to_flat_item(
        &self,
        account: &CoaAccount::Model,
    ) -> Result<ChartOfAccountFlatItem, AppError> {
        // Expose public ids rather than internal primary ids so the response
        // is stable and safe for clients to store.
        Ok(ChartOfAccountFlatItem {
            public_id: account.public_id.clone(),
            code: account.code.clone(),
            name_primary: account.name_primary.clone(),
            name_secondary: account.name_secondary.clone(),
            description: account.description.clone(),
            level_no: account.level_no,
            is_posting: account.is_posting,
            is_system_account: account.is_system_account,
            parent_public_id: resolve_public_id(
                account.parent_account_id,
                &self.accounts_by_id,
                "parent account",
            )?,
            account_group_public_id: resolve_public_id(
                account.account_group_id,
                &self.accounts_by_id,
                "account group",
            )?,
            account_type_public_id: resolve_public_id(
                account.account_type_id,
                &self.accounts_by_id,
                "account type",
            )?,
        })
    }

    fn to_tree_item(
        &self,
        account: &CoaAccount::Model,
        children: Vec<ChartOfAccountTreeItem>,
    ) -> Result<ChartOfAccountTreeItem, AppError> {
        // Tree nodes carry the same account metadata as flat nodes plus the
        // nested children vector.
        Ok(ChartOfAccountTreeItem {
            public_id: account.public_id.clone(),
            code: account.code.clone(),
            name_primary: account.name_primary.clone(),
            name_secondary: account.name_secondary.clone(),
            description: account.description.clone(),
            level_no: account.level_no,
            is_posting: account.is_posting,
            is_system_account: account.is_system_account,
            parent_public_id: resolve_public_id(
                account.parent_account_id,
                &self.accounts_by_id,
                "parent account",
            )?,
            account_group_public_id: resolve_public_id(
                account.account_group_id,
                &self.accounts_by_id,
                "account group",
            )?,
            account_type_public_id: resolve_public_id(
                account.account_type_id,
                &self.accounts_by_id,
                "account type",
            )?,
            children,
        })
    }
}

fn ensure_account_exists(
    account_id: PrimaryId,
    accounts_by_id: &HashMap<PrimaryId, CoaAccount::Model>,
    label: &str,
    seed_account_id: PrimaryId,
) -> Result<(), AppError> {
    if accounts_by_id.contains_key(&account_id) {
        Ok(())
    } else {
        Err(AppError::InternalServer(format!(
            "Missing {label} `{account_id}` while resolving COA account `{seed_account_id}`"
        )))
    }
}

fn resolve_public_id(
    account_id: Option<PrimaryId>,
    accounts_by_id: &HashMap<PrimaryId, CoaAccount::Model>,
    label: &str,
) -> Result<Option<PublicId>, AppError> {
    match account_id {
        Some(account_id) => accounts_by_id
            .get(&account_id)
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
