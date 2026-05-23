use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::auth::permission::{Permission, normalize_permission_codes, serialize_permission_codes};
use crate::entity::organization::staff_role_entity as StaffRole;
use crate::entity::{ActorPrimaryId, OrganizationPrimaryId, PublicId};
use crate::errors::app_error::AppError;
use crate::errors::staff_service_errors::StaffServiceError;
use crate::service::service_context::ServiceContext;
use crate::utils::date_helpers::DateHelper;
use crate::utils::id_generator::IdGenerator;

pub struct StaffRoleService;

#[derive(Deserialize)]
pub struct CreateStaffRole {
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub permission_codes: Vec<String>,
}

pub struct DefaultOrganizationRoles {
    pub owner_role_id: i32,
}

impl StaffRoleService {
    pub async fn create_staff_role(
        ctx: &ServiceContext,
        payload: CreateStaffRole,
    ) -> Result<PublicId, AppError> {
        let actor_id = ctx.get_actor_id()?;
        let organization_id = ctx.get_organization_id()?;
        let permission_codes = normalize_permission_codes(&payload.permission_codes)?;
        let role = Self::create_role(
            &ctx.app_state.primary_write_replica,
            actor_id,
            organization_id,
            payload.name_primary,
            payload.name_secondary,
            &permission_codes,
            false,
        )
        .await?;

        Ok(role.public_id)
    }

    pub async fn create_default_roles_for_organization(
        db_transaction: &impl ConnectionTrait,
        actor_id: ActorPrimaryId,
        organization_id: OrganizationPrimaryId,
    ) -> Result<DefaultOrganizationRoles, AppError> {
        // Bootstrap a standard role set per organization so authorization can
        // stay data-driven while new organizations still start in a usable state.
        let owner_role = Self::create_role(
            db_transaction,
            actor_id,
            organization_id,
            "Owner".to_string(),
            None,
            &Permission::all_codes(),
            true,
        )
        .await?;

        Self::create_role(
            db_transaction,
            actor_id,
            organization_id,
            "Admin".to_string(),
            None,
            &vec![
                Permission::BranchCreate.code().to_string(),
                Permission::StaffInvite.code().to_string(),
                Permission::StaffInvitationResend.code().to_string(),
                Permission::StaffInvitationRevoke.code().to_string(),
                Permission::StaffRoleCreate.code().to_string(),
            ],
            true,
        )
        .await?;

        Self::create_role(
            db_transaction,
            actor_id,
            organization_id,
            "Manager".to_string(),
            None,
            &vec![
                Permission::BranchCreate.code().to_string(),
                Permission::StaffInvite.code().to_string(),
                Permission::StaffInvitationResend.code().to_string(),
                Permission::StaffInvitationRevoke.code().to_string(),
            ],
            true,
        )
        .await?;

        Self::create_role(
            db_transaction,
            actor_id,
            organization_id,
            "Staff".to_string(),
            None,
            &Vec::new(),
            true,
        )
        .await?;

        Ok(DefaultOrganizationRoles {
            owner_role_id: owner_role.id,
        })
    }

    pub async fn get_role_by_public_id_for_organization(
        db_transaction: &impl ConnectionTrait,
        organization_id: OrganizationPrimaryId,
        role_public_id: &str,
    ) -> Result<StaffRole::Model, AppError> {
        StaffRole::Entity::find()
            .filter(StaffRole::Column::OrganizationId.eq(organization_id))
            .filter(StaffRole::Column::PublicId.eq(role_public_id))
            .one(db_transaction)
            .await?
            .ok_or(StaffServiceError::RoleNotFound.into())
    }

    async fn create_role(
        db_transaction: &impl ConnectionTrait,
        actor_id: ActorPrimaryId,
        organization_id: OrganizationPrimaryId,
        name_primary: String,
        name_secondary: Option<String>,
        permission_codes: &[String],
        is_system_role: bool,
    ) -> Result<StaffRole::Model, AppError> {
        let now = DateHelper::now().value();
        StaffRole::ActiveModel {
            organization_id: Set(organization_id),
            public_id: Set(IdGenerator::generate_general_id()),
            name_primary: Set(name_primary),
            name_secondary: Set(name_secondary),
            permissions: Set(serialize_permission_codes(permission_codes)?),
            is_system_role: Set(is_system_role),
            created_by_actor_id: Set(actor_id),
            updated_by_actor_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(db_transaction)
        .await
        .map_err(Into::into)
    }
}
