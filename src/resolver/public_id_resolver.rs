use std::collections::HashSet;

use sea_orm::ConnectionTrait;

use crate::entity::organization::{
    branch_entity as Branch, organization_entity as Organization,
    organization_meta_entity as OrganizationMeta, staff_invitation_entity as StaffInvitation,
    staff_role_entity as StaffRole,
};
use crate::entity::{BranchPrimaryId, OrganizationPrimaryId, actor_entity as Actor};
use crate::errors::app_error::AppError;
use crate::errors::organization_service_errors::OrgServiceError;
use crate::errors::staff_service_errors::StaffServiceError;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

// Centralizes public-id to internal-id/model resolution so handlers can keep
// transport-facing ids at the edge and services can operate on trusted DB ids.
pub struct PublicIdResolver;

impl PublicIdResolver {
    pub async fn organization(
        db_transaction: &impl ConnectionTrait,
        public_id: &str,
    ) -> Result<Organization::Model, AppError> {
        Organization::Entity::find()
            .filter(Organization::Column::PublicId.eq(public_id))
            .one(db_transaction)
            .await?
            .ok_or(OrgServiceError::NotFound.into())
    }

    pub async fn staff_invitation(
        db_transaction: &impl ConnectionTrait,
        public_id: &str,
    ) -> Result<StaffInvitation::Model, AppError> {
        StaffInvitation::Entity::find()
            .filter(StaffInvitation::Column::PublicId.eq(public_id))
            .one(db_transaction)
            .await?
            .ok_or(StaffServiceError::InvitationNotFound.into())
    }

    pub async fn staff_role_for_organization(
        db_transaction: &impl ConnectionTrait,
        organization_id: OrganizationPrimaryId,
        public_id: &str,
    ) -> Result<StaffRole::Model, AppError> {
        StaffRole::Entity::find()
            .filter(StaffRole::Column::OrganizationId.eq(organization_id))
            .filter(StaffRole::Column::PublicId.eq(public_id))
            .one(db_transaction)
            .await?
            .ok_or(StaffServiceError::RoleNotFound.into())
    }

    pub async fn branch_ids_for_organization(
        db_transaction: &impl ConnectionTrait,
        organization_id: OrganizationPrimaryId,
        branch_public_ids: Option<&[String]>,
    ) -> Result<Vec<BranchPrimaryId>, AppError> {
        let Some(branch_public_ids) = branch_public_ids else {
            return Ok(vec![
                Self::default_branch_id_for_organization(db_transaction, organization_id).await?,
            ]);
        };

        let normalized_branch_public_ids = normalize_public_ids(branch_public_ids);
        if normalized_branch_public_ids.is_empty() {
            return Ok(vec![
                Self::default_branch_id_for_organization(db_transaction, organization_id).await?,
            ]);
        }

        let branches = Branch::Entity::find()
            .filter(Branch::Column::OrganizationId.eq(organization_id))
            .filter(Branch::Column::PublicId.is_in(normalized_branch_public_ids.iter().cloned()))
            .all(db_transaction)
            .await?;

        if branches.len() != normalized_branch_public_ids.len() {
            return Err(OrgServiceError::BranchNotFound.into());
        }

        Ok(branches.into_iter().map(|branch| branch.id).collect())
    }

    pub async fn user_actor(
        db_transaction: &impl ConnectionTrait,
        public_id: &str,
    ) -> Result<Actor::Model, AppError> {
        Actor::Entity::find()
            .filter(Actor::Column::PublicUserId.eq(public_id))
            .one(db_transaction)
            .await
            .map_err(AppError::Database)?
            .ok_or(AppError::ActorIdNotFound)
    }

    async fn default_branch_id_for_organization(
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
}

fn normalize_public_ids(public_ids: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();

    public_ids
        .iter()
        .map(|public_id| public_id.trim())
        .filter(|public_id| !public_id.is_empty())
        .filter(|public_id| seen.insert((*public_id).to_owned()))
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::normalize_public_ids;

    #[test]
    fn normalize_public_ids_trims_filters_and_deduplicates() {
        let public_ids = vec![
            " branch_1 ".to_string(),
            "".to_string(),
            "branch_1".to_string(),
            "branch_2".to_string(),
            "   ".to_string(),
        ];

        let normalized = normalize_public_ids(&public_ids);

        assert_eq!(
            normalized,
            vec!["branch_1".to_string(), "branch_2".to_string()]
        );
    }
}
