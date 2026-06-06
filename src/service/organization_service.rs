use sea_orm::{ActiveModelTrait, ConnectionTrait, Set, TransactionTrait};

use crate::entity::GenericStatus;
use crate::entity::organization::organization_entity::OrganizationStatus;
use crate::{
    entity::{
        PrimaryId, PublicId,
        organization::{
            organization_entity as Organization, organization_meta_entity as OrganizationMeta,
        },
    },
    errors::app_error::AppError,
    service::branch_service::BranchService,
    service::service_context::ServiceContext,
    service::staff_role_service::StaffRoleService,
    service::staff_service::StaffService,
    service::unit_service::UnitService,
    utils::{date_helpers::DateHelper, id_generator::IdGenerator},
};

pub struct CreateOrganizationInput {
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub country_iso_code: String,
    pub currency_iso_code: String,
}

struct CreateOrganizationMeta {
    pub country_iso_code: String,
    pub currency_iso_code: String,
}

impl From<&CreateOrganizationInput> for CreateOrganizationMeta {
    fn from(value: &CreateOrganizationInput) -> Self {
        Self {
            country_iso_code: value.country_iso_code.clone(),
            currency_iso_code: value.currency_iso_code.clone(),
        }
    }
}

pub struct OrganizationService;

impl OrganizationService {
    // Create the organization record first, then hand off all default
    // organization bootstrapping to dedicated helpers inside the same transaction.
    pub async fn create_organization(
        ctx: &ServiceContext,
        payload: CreateOrganizationInput,
    ) -> Result<PublicId, AppError> {
        let actor_id = ctx.get_actor_id()?;
        let user_id = ctx.get_user_id()?;

        let public_id = IdGenerator::get_organization_id();
        let now = DateHelper::now().value();

        let meta_payload = CreateOrganizationMeta::from(&payload);

        let organization_active_model = Organization::ActiveModel {
            status: Set(OrganizationStatus::Active),
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
        // create organization related things lik creating default branch, items, units etc.
        Self::seed_organization_defaults(&txn, ctx, actor_id, created_organization.id).await?;

        txn.commit().await?;

        Ok(created_organization.public_id)
    }

    // Seed the minimum organization data required for staff, branch, role,
    // and item flows to work immediately after organization creation.
    async fn seed_organization_defaults(
        db_transaction: &impl ConnectionTrait,
        ctx: &ServiceContext,
        actor_id: PrimaryId,
        organization_id: PrimaryId,
    ) -> Result<(), AppError> {
        // create organization default roles.
        let default_roles =
            StaffRoleService::seed_default_roles(db_transaction, actor_id, organization_id).await?;
        // Every organization starts with a primary branch so downstream staff flows
        // always have a valid default branch to attach people to.
        let default_branch = BranchService::create_branch_for_organization(
            db_transaction,
            actor_id,
            organization_id,
            "Head Office".to_string(),
            None,
            true,
        )
        .await?;
        UnitService::seed_default_units_for_organization(db_transaction, actor_id, organization_id)
            .await?;
        // register this user as organization staff
        StaffService::create_staff_from_user(
            db_transaction,
            ctx,
            organization_id,
            &[default_branch.id],
            default_roles.owner_role_id,
        )
        .await?;

        Ok(())
    }

    // Persist organization-scoped metadata separately so the main organization
    // row stays focused on identity while defaults can evolve independently.
    async fn create_organization_meta(
        db_transaction: &impl ConnectionTrait,
        actor_id: PrimaryId,
        organization_id: PrimaryId,
        payload: CreateOrganizationMeta,
    ) -> Result<(), AppError> {
        let now = DateHelper::now().value();
        let organization_meta_active_model = OrganizationMeta::ActiveModel {
            status: Set(GenericStatus::Active),
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
