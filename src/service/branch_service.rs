use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};

use crate::entity::organization::{
    branch_entity as Branch, organization_meta_entity as OrganizationMeta,
};
use crate::entity::{ActorPrimaryId, BranchPrimaryId, OrganizationPrimaryId, PublicId};
use crate::errors::app_error::AppError;
use crate::errors::organization_service_errors::OrgServiceError;
use crate::service::service_context::ServiceContext;
use crate::utils::date_helpers::DateHelper;
use crate::utils::id_generator::IdGenerator;

#[derive(Deserialize, Serialize)]
pub struct CreateBranch {
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub is_primary: Option<bool>,
}

pub struct BranchService;

impl BranchService {
    pub async fn get_branch_by_public_id(
        db_transaction: &impl ConnectionTrait,
        public_id: &str,
    ) -> Result<Branch::Model, AppError> {
        let branch = Branch::Entity::find()
            .filter(Branch::COLUMN.public_id.eq(public_id))
            .one(db_transaction)
            .await?
            .ok_or(OrgServiceError::BranchNotFound)?;
        Ok(branch)
    }

    pub async fn create_branch(
        ctx: &ServiceContext,
        payload: CreateBranch,
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

    pub async fn create_branch_for_organization(
        db_transaction: &impl ConnectionTrait,
        actor_id: ActorPrimaryId,
        organization_id: OrganizationPrimaryId,
        name_primary: String,
        name_secondary: Option<String>,
        is_primary_requested: bool,
    ) -> Result<Branch::Model, AppError> {
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

    pub async fn resolve_branch_ids_for_organization(
        db_transaction: &impl ConnectionTrait,
        organization_id: OrganizationPrimaryId,
        branch_public_ids: Option<&[String]>,
    ) -> Result<Vec<BranchPrimaryId>, AppError> {
        let Some(branch_public_ids) = branch_public_ids else {
            return Ok(vec![
                Self::get_default_branch_id_for_organization(db_transaction, organization_id)
                    .await?,
            ]);
        };

        let normalized_branch_public_ids = branch_public_ids
            .iter()
            .map(|branch_public_id| branch_public_id.trim())
            .filter(|branch_public_id| !branch_public_id.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();

        // Empty or whitespace-only branch ids should behave the same as omitting the field:
        // assign the organization's configured default branch.
        if normalized_branch_public_ids.is_empty() {
            return Ok(vec![
                Self::get_default_branch_id_for_organization(db_transaction, organization_id)
                    .await?,
            ]);
        }

        let branches = Branch::Entity::find()
            .filter(Branch::COLUMN.organization_id.eq(organization_id))
            .filter(
                Branch::COLUMN
                    .public_id
                    .is_in(normalized_branch_public_ids.iter().cloned()),
            )
            .all(db_transaction)
            .await?;

        if branches.len() != normalized_branch_public_ids.len() {
            return Err(OrgServiceError::BranchNotFound.into());
        }

        Ok(branches.into_iter().map(|branch| branch.id).collect())
    }

    pub async fn get_default_branch_id_for_organization(
        db_transaction: &impl ConnectionTrait,
        organization_id: OrganizationPrimaryId,
    ) -> Result<BranchPrimaryId, AppError> {
        let organization_meta = OrganizationMeta::Entity::find_by_id(organization_id)
            .one(db_transaction)
            .await?
            .ok_or(OrgServiceError::NotFound)?;

        organization_meta
            .default_branch_id
            .ok_or(OrgServiceError::PrimaryBranchNotConfigured.into())
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
}
