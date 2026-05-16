use sea_orm::{ActiveModelTrait, DbErr, Set};
use serde::Deserialize;

use crate::{
    entity::{
        ActorPrimaryId, OrganizationPrimaryId,
        organization::{
            organization_entity::{self as Organization},
            organization_meta_entity::{self as OrganizationMeta},
        },
    },
    errors::app_error::AppError,
    service::service_context::ServiceContext,
    utils::{date_helpers::DateHelper, id_generator::IdGenerator},
};

#[derive(Deserialize)]
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

struct OrganizationService;

impl OrganizationService {
    pub async fn create_organization(
        ctx: &ServiceContext,
        payload: CreateOrganization,
    ) -> Result<(), AppError> {
        let actor_id = ctx.get_actor_id()?;

        let public_id = IdGenerator::get_organization_id();
        let now = DateHelper::now().value();

        let meta_payload = CreateOrganizationMeta::from(&payload);

        let organization_active_model = Organization::ActiveModel {
            user_id: Set(None),
            public_id: Set(public_id),
            name_primary: Set(payload.name_primary),
            name_secondary: Set(payload.name_secondary),
            created_by_actor_id: Set(actor_id),
            updated_by_actor_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        // create organization
        let created_organization = organization_active_model
            .insert(&ctx.app_state.primary_write_replica)
            .await?;
        let organization_id = created_organization.id;

        // create organization meta data.
        Self::create_organization_meta(ctx, organization_id, meta_payload).await?;

        Ok(())
    }

    async fn create_organization_meta(
        ctx: &ServiceContext,
        organization_id: OrganizationPrimaryId,
        payload: CreateOrganizationMeta,
    ) -> Result<(), AppError> {
        let actor_id = ctx.get_actor_id()?;
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
            .insert(&ctx.app_state.primary_write_replica)
            .await?;

        Ok(())
    }
}
