use std::collections::HashSet;

use sea_orm::ConnectionTrait;

use crate::entity::item::unit_entity as Unit;
use crate::entity::organization::{
    branch_entity as Branch, organization_entity as Organization,
    organization_meta_entity as OrganizationMeta,
};
use crate::entity::staff::{
    staff_invitation_entity as StaffInvitation, staff_role_entity as StaffRole,
};
use crate::entity::{PrimaryId, actor_entity as Actor};
use crate::errors::app_error::AppError;
use crate::errors::item_service_errors::ItemServiceError;
use crate::errors::organization_service_errors::OrgServiceError;
use crate::errors::staff_service_errors::StaffServiceError;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use crate::errors::branch_service_errors::BranchServiceError;

// Centralizes public-id to internal-id/model resolution so handlers can keep
// transport-facing ids at the edge and services can operate on trusted DB ids.
pub struct PublicIdResolver;

impl PublicIdResolver {
    pub async fn organization_id(
        db_transaction: &impl ConnectionTrait,
        public_id: &str,
    ) -> Result<PrimaryId, AppError> {
        Organization::Entity::find()
            .filter(Organization::Column::PublicId.eq(public_id))
            .one(db_transaction)
            .await?
            .map(|organization| organization.id)
            .ok_or(OrgServiceError::NotFound.into())
    }

    pub async fn staff_invitation_id(
        db_transaction: &impl ConnectionTrait,
        organization_id: PrimaryId,
        public_id: &str,
    ) -> Result<PrimaryId, AppError> {
        StaffInvitation::Entity::find()
            .filter(StaffInvitation::Column::OrganizationId.eq(organization_id))
            .filter(StaffInvitation::Column::PublicId.eq(public_id))
            .one(db_transaction)
            .await?
            .map(|staff_invitation| staff_invitation.id)
            .ok_or(StaffServiceError::InvitationNotFound.into())
    }

    pub async fn staff_role_id(
        db_transaction: &impl ConnectionTrait,
        organization_id: PrimaryId,
        public_id: &str,
    ) -> Result<PrimaryId, AppError> {
        StaffRole::Entity::find()
            .filter(StaffRole::Column::OrganizationId.eq(organization_id))
            .filter(StaffRole::Column::PublicId.eq(public_id))
            .one(db_transaction)
            .await?
            .map(|staff_role| staff_role.id)
            .ok_or(StaffServiceError::RoleNotFound.into())
    }

    pub async fn branch_ids(
        db_transaction: &impl ConnectionTrait,
        organization_id: PrimaryId,
        branch_public_ids: Option<&[String]>,
    ) -> Result<Vec<PrimaryId>, AppError> {
        let normalized_branch_public_ids = branch_public_ids
            .map(|public_ids| normalize_public_ids(public_ids, true))
            .unwrap_or_default();
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
            return Err(BranchServiceError::NotFound.into());
        }

        Ok(branches.into_iter().map(|branch| branch.id).collect())
    }

    pub async fn actor_id(
        db_transaction: &impl ConnectionTrait,
        public_id: &str,
    ) -> Result<PrimaryId, AppError> {
        Actor::Entity::find()
            .filter(Actor::Column::PublicUserId.eq(public_id))
            .one(db_transaction)
            .await
            .map_err(AppError::Database)?
            .map(|actor| actor.id)
            .ok_or(AppError::ActorIdNotFound)
    }

    pub async fn unit_ids(
        db_transaction: &impl ConnectionTrait,
        organization_id: PrimaryId,
        unit_public_ids: &[String],
    ) -> Result<Vec<PrimaryId>, AppError> {
        let normalized_unit_public_ids = normalize_public_ids(unit_public_ids, false);
        if normalized_unit_public_ids.is_empty() {
            return Err(ItemServiceError::UnitNotFound.into());
        }

        let unique_unit_public_ids = normalize_public_ids(unit_public_ids, true);
        let units = Unit::Entity::find()
            .filter(Unit::Column::OrganizationId.eq(organization_id))
            .filter(Unit::Column::PublicId.is_in(unique_unit_public_ids.iter().cloned()))
            .all(db_transaction)
            .await?;

        let units_by_public_id = units
            .into_iter()
            .map(|unit| (unit.public_id.clone(), unit.id))
            .collect::<std::collections::HashMap<_, _>>();

        if units_by_public_id.len() != unique_unit_public_ids.len() {
            return Err(ItemServiceError::UnitNotFound.into());
        }

        normalized_unit_public_ids
            .iter()
            .map(|public_id| {
                units_by_public_id
                    .get(public_id)
                    .copied()
                    .ok_or(ItemServiceError::UnitNotFound.into())
            })
            .collect()
    }

    async fn default_branch_id_for_organization(
        db_transaction: &impl ConnectionTrait,
        organization_id: PrimaryId,
    ) -> Result<PrimaryId, AppError> {
        let organization_meta = OrganizationMeta::Entity::find_by_id(organization_id)
            .one(db_transaction)
            .await?
            .ok_or(OrgServiceError::NotFound)?;

        organization_meta
            .default_branch_id
            .ok_or(OrgServiceError::PrimaryBranchNotConfigured.into())
    }
}

fn normalize_public_ids(public_ids: &[String], deduplicate: bool) -> Vec<String> {
    let normalized_public_ids = public_ids
        .iter()
        .map(|public_id| public_id.trim())
        .filter(|public_id| !public_id.is_empty())
        .map(str::to_owned);

    if !deduplicate {
        return normalized_public_ids.collect();
    }

    let mut seen = HashSet::new();
    normalized_public_ids
        .filter(|public_id| seen.insert(public_id.clone()))
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

        let normalized = normalize_public_ids(&public_ids, true);

        assert_eq!(
            normalized,
            vec!["branch_1".to_string(), "branch_2".to_string()]
        );
    }

    #[test]
    fn normalize_public_ids_preserves_duplicates_when_requested() {
        let public_ids = vec![
            " unit_1 ".to_string(),
            "".to_string(),
            "unit_1".to_string(),
            "unit_2".to_string(),
        ];

        let normalized = normalize_public_ids(&public_ids, false);

        assert_eq!(
            normalized,
            vec![
                "unit_1".to_string(),
                "unit_1".to_string(),
                "unit_2".to_string()
            ]
        );
    }
}
