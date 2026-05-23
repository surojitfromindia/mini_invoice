use sea_orm::FromQueryResult;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set, TransactionTrait,
};

use crate::db::listing::PageListResult;
use crate::db::listing::{execute_page_query, validate_page_pagination};
use crate::entity::organization::branch_entity::BranchModel;
use crate::entity::organization::{
    branch_entity as Branch, organization_meta_entity as OrganizationMeta,
};
use crate::entity::{ActorPrimaryId, BranchPrimaryId, OrganizationPrimaryId, PublicId};
use crate::errors::app_error::AppError;
use crate::service::service_context::ServiceContext;
use crate::utils::date_helpers::DateHelper;
use crate::utils::id_generator::IdGenerator;

pub struct CreateBranchInput {
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub is_primary: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchSortField {
    CreatedAt,
    NamePrimary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BranchInclude {
    Organization,
}

pub struct BranchListPageInput {
    pub page: u64,
    pub per_page: u64,
    pub name: Option<String>,
    pub is_primary: Option<bool>,
    pub sort: Option<BranchSortField>,
    pub direction: Option<SortDirection>,
    pub include: Vec<BranchInclude>,
}

#[derive(Debug, Clone, PartialEq, Eq, FromQueryResult)]
pub struct BranchListItem {
    pub public_id: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub is_primary: bool,
}

pub struct BranchService;

impl BranchService {
    pub async fn create_branch(
        ctx: &ServiceContext,
        payload: CreateBranchInput,
    ) -> Result<PublicId, AppError> {
        let actor_id = ctx.get_actor_id()?;
        let organization_id = ctx.get_organization_id()?;
        let txn = ctx.app_state.primary_write_replica.begin().await?;

        let branch = Self::create_branch_for_organization(
            &txn,
            actor_id,
            organization_id,
            payload.name_primary,
            payload.name_secondary,
            payload.is_primary.unwrap_or(false),
        )
        .await?;

        txn.commit().await?;

        Ok(branch.public_id)
    }

    pub async fn list_branches_page(
        ctx: &ServiceContext,
        input: BranchListPageInput,
    ) -> Result<PageListResult<BranchListItem>, AppError> {
        let pagination = validate_page_pagination(input.page, input.per_page)?;
        let organization_id = ctx.get_organization_id()?;
        let sort_field = input.sort.unwrap_or(BranchSortField::CreatedAt);
        let sort_direction = input.direction.unwrap_or(SortDirection::Desc);

        let query =
            Self::build_branch_list_query(organization_id, input.name.as_deref(), input.is_primary);
        let query = Self::apply_page_sort(query, sort_field, sort_direction);
        let query = Self::select_branch_list_columns(query).into_model::<BranchListItem>();

        let result =
            execute_page_query(&ctx.app_state.primary_read_replica, query, pagination).await?;

        Ok(result)
    }

    pub async fn create_branch_for_organization(
        db_transaction: &impl ConnectionTrait,
        actor_id: ActorPrimaryId,
        organization_id: OrganizationPrimaryId,
        name_primary: String,
        name_secondary: Option<String>,
        is_primary_requested: bool,
    ) -> Result<BranchModel, AppError> {
        let now = DateHelper::now().value();
        let existing_branch = Branch::Entity::find()
            .filter(Branch::COLUMN.organization_id.eq(organization_id))
            .one(db_transaction)
            .await?;
        let should_be_primary = is_primary_requested || existing_branch.is_none();

        let branch = Branch::ActiveModel {
            organization_id: Set(organization_id),
            public_id: Set(IdGenerator::generate_general_id()),
            name_primary: Set(name_primary),
            name_secondary: Set(name_secondary),
            is_primary: Set(should_be_primary),
            created_by_actor_id: Set(actor_id),
            updated_by_actor_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(db_transaction)
        .await?;

        if should_be_primary {
            Self::set_primary_branch(db_transaction, actor_id, organization_id, branch.id).await?;
        }

        Ok(branch)
    }

    async fn set_primary_branch(
        db_transaction: &impl ConnectionTrait,
        actor_id: ActorPrimaryId,
        organization_id: OrganizationPrimaryId,
        branch_id: BranchPrimaryId,
    ) -> Result<(), AppError> {
        let now = DateHelper::now().value();

        Branch::Entity::update_many()
            .col_expr(
                Branch::COLUMN.is_primary,
                sea_orm::sea_query::Expr::value(false),
            )
            .col_expr(
                Branch::COLUMN.updated_by_actor_id,
                sea_orm::sea_query::Expr::value(Some(actor_id)),
            )
            .col_expr(
                Branch::COLUMN.updated_at,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(Branch::COLUMN.organization_id.eq(organization_id))
            .filter(Branch::COLUMN.id.ne(branch_id))
            .exec(db_transaction)
            .await?;

        Branch::Entity::update_many()
            .col_expr(
                Branch::COLUMN.is_primary,
                sea_orm::sea_query::Expr::value(true),
            )
            .col_expr(
                Branch::COLUMN.updated_by_actor_id,
                sea_orm::sea_query::Expr::value(Some(actor_id)),
            )
            .col_expr(
                Branch::COLUMN.updated_at,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(Branch::COLUMN.id.eq(branch_id))
            .exec(db_transaction)
            .await?;

        OrganizationMeta::Entity::update_many()
            .col_expr(
                OrganizationMeta::COLUMN.default_branch_id,
                sea_orm::sea_query::Expr::value(Some(branch_id)),
            )
            .col_expr(
                OrganizationMeta::COLUMN.updated_by_actor_id,
                sea_orm::sea_query::Expr::value(Some(actor_id)),
            )
            .col_expr(
                OrganizationMeta::COLUMN.updated_at,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(OrganizationMeta::COLUMN.organization_id.eq(organization_id))
            .exec(db_transaction)
            .await?;

        Ok(())
    }

    fn build_branch_list_query(
        organization_id: OrganizationPrimaryId,
        name: Option<&str>,
        is_primary: Option<bool>,
    ) -> sea_orm::Select<Branch::Entity> {
        let mut query =
            Branch::Entity::find().filter(Branch::COLUMN.organization_id.eq(organization_id));

        if let Some(name) = name.map(str::trim).filter(|value| !value.is_empty()) {
            query = query.filter(Branch::COLUMN.name_primary.contains(name));
        }

        if let Some(is_primary) = is_primary {
            query = query.filter(Branch::COLUMN.is_primary.eq(is_primary));
        }

        query
    }

    fn apply_page_sort(
        query: sea_orm::Select<Branch::Entity>,
        sort_field: BranchSortField,
        sort_direction: SortDirection,
    ) -> sea_orm::Select<Branch::Entity> {
        match (sort_field, sort_direction) {
            (BranchSortField::CreatedAt, SortDirection::Asc) => query
                .order_by_asc(Branch::Column::CreatedAt)
                .order_by_asc(Branch::Column::Id),
            (BranchSortField::CreatedAt, SortDirection::Desc) => query
                .order_by_desc(Branch::Column::CreatedAt)
                .order_by_desc(Branch::Column::Id),
            (BranchSortField::NamePrimary, SortDirection::Asc) => query
                .order_by_asc(Branch::Column::NamePrimary)
                .order_by_asc(Branch::Column::Id),
            (BranchSortField::NamePrimary, SortDirection::Desc) => query
                .order_by_desc(Branch::Column::NamePrimary)
                .order_by_desc(Branch::Column::Id),
        }
    }

    fn select_branch_list_columns<Q>(query: Q) -> Q
    where
        Q: QuerySelect<QueryStatement = sea_orm::sea_query::SelectStatement>,
    {
        query
            .select_only()
            .column(Branch::Column::PublicId)
            .column(Branch::Column::NamePrimary)
            .column(Branch::Column::NameSecondary)
            .column(Branch::Column::IsPrimary)
    }
}
