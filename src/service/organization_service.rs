use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};

use crate::entity::{ActorPrimaryId, PublicId};
use crate::errors::organization_service_errors::OrgServiceError;
use crate::service::branch_service::BranchService;
use crate::service::staff_role_service::StaffRoleService;
use crate::service::staff_service::StaffService;
use crate::{
    entity::{
        OrganizationPrimaryId,
        organization::{
            organization_entity as Organization, organization_meta_entity as OrganizationMeta,
        },
    },
    errors::app_error::AppError,
    service::service_context::ServiceContext,
    utils::{date_helpers::DateHelper, id_generator::IdGenerator},
};

#[derive(Deserialize, Serialize)]
pub struct CreateOrganization {
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub country_iso_code: String,
    pub currency_iso_code: String,
}

struct CreateOrganizationMeta {
    pub country_iso_code: String,
    pub currency_iso_code: String,
}

impl From<&CreateOrganization> for CreateOrganizationMeta {
    fn from(value: &CreateOrganization) -> Self {
        Self {
            country_iso_code: value.country_iso_code.clone(),
            currency_iso_code: value.currency_iso_code.clone(),
        }
    }
}

pub struct OrganizationService;

impl OrganizationService {
    pub async fn create_organization(
        ctx: &ServiceContext,
        payload: CreateOrganization,
    ) -> Result<PublicId, AppError> {
        let actor_id = ctx.get_actor_id()?;
        let user_id = ctx.get_user_id()?;

        let public_id = IdGenerator::get_organization_id();
        let now = DateHelper::now().value();

        let meta_payload = CreateOrganizationMeta::from(&payload);

        let organization_active_model = Organization::ActiveModel {
            prime_user_id: Set(user_id),
            public_id: Set(public_id),
            name_primary: Set(payload.name_primary),
            name_secondary: Set(payload.name_secondary),
            created_by_actor_id: Set(actor_id),
            updated_by_actor_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let txn = ctx.app_state.primary_write_replica.begin().await?;

        // create organization
        let created_organization = organization_active_model.insert(&txn).await?;
        // create organization meta data.
        Self::create_organization_meta(&txn, actor_id, created_organization.id, meta_payload)
            .await?;
        let default_roles = StaffRoleService::create_default_roles_for_organization(
            &txn,
            actor_id,
            created_organization.id,
        )
        .await?;
        // Every organization starts with a primary branch so downstream staff flows
        // always have a valid default branch to attach people to.
        let default_branch = BranchService::create_branch_for_organization(
            &txn,
            actor_id,
            created_organization.id,
            "Head Office".to_string(),
            None,
            true,
        )
        .await?;
        // register this user as organization staff
        StaffService::create_staff_from_user(
            &txn,
            &ctx,
            created_organization.id,
            &[default_branch.id],
            default_roles.owner_role_id,
        )
        .await?;

        txn.commit().await?;

        Ok(created_organization.public_id)
    }

    async fn create_organization_meta(
        db_transaction: &impl ConnectionTrait,
        actor_id: ActorPrimaryId,
        organization_id: OrganizationPrimaryId,
        payload: CreateOrganizationMeta,
    ) -> Result<(), AppError> {
        let now = DateHelper::now().value();
        let organization_meta_active_model = OrganizationMeta::ActiveModel {
            organization_id: Set(organization_id),
            country_iso_code: Set(payload.country_iso_code),
            currency_iso_code: Set(payload.currency_iso_code),
            default_branch_id: Set(None),
            created_by_actor_id: Set(actor_id),
            updated_by_actor_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        organization_meta_active_model
            .insert(db_transaction)
            .await?;

        Ok(())
    }
}
