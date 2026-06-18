use std::collections::{HashMap, HashSet};

use crate::config::settings::Settings;
use crate::db::listing::{PageListResult, execute_page_query, validate_page_pagination};
use crate::entity::organization::branch_entity::{self as Branch, BranchStatus};
use crate::entity::organization::organization_entity as Organization;
use crate::entity::organization::organization_entity::OrganizationModel;
use crate::entity::staff::staff_branch_entity as StaffBranch;
use crate::entity::staff::staff_entity::{self as Staff, StaffStatus};
use crate::entity::staff::staff_invitation_branch_entity as StaffInvitationBranch;
use crate::entity::staff::staff_invitation_entity::{
    self as StaffInvitation, StaffInvitationStatus,
};
use crate::entity::staff::staff_role_entity as StaffRole;
use crate::entity::user_entity::{self as User, UserModel};
use crate::entity::{GenericStatus, PrimaryId, PublicId};
use crate::errors::app_error::AppError;
use crate::errors::staff_service_errors::StaffServiceError;
use crate::service::service_context::ServiceContext;
use crate::service::user_credential_service::UserCredentialService;
use crate::service::user_service::UserService;
use crate::utils::date_helpers::DateHelper;
use crate::utils::id_generator::IdGenerator;
use crate::utils::misc_helpers::trim_and_filter_empty;
use crate::utils::password_helpers::PasswordHelpers;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseTransaction, EntityTrait,
    IntoActiveModel, QueryFilter, QueryOrder, Set, TransactionTrait,
};

pub struct StaffService;

pub struct CreateStaffInvitationInput {
    pub invitee_email: String,
    pub invitee_first_name: String,
    pub invitee_last_name: String,
    pub invited_role_id: PrimaryId,
    pub branch_ids: Vec<PrimaryId>,
}

pub struct AcceptStaffInvitationInput {
    pub invitation_token: String,
    pub password: String,
}

pub struct UpdateStaffInput {
    pub name_primary: Option<String>,
    pub name_secondary: Option<String>,
    pub role_id: Option<PrimaryId>,
    pub branch_ids: Option<Vec<PrimaryId>>,
    pub is_default_organization: Option<bool>,
    pub status: Option<StaffStatus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StaffSortField {
    CreatedAt,
    NamePrimary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

pub struct StaffListPageInput {
    pub page: u64,
    pub per_page: u64,
    pub name: Option<String>,
    pub status: Option<StaffStatus>,
    pub sort: Option<StaffSortField>,
    pub direction: Option<SortDirection>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaffListItem {
    pub public_id: PublicId,
    pub user_public_id: PublicId,
    pub user_email: String,
    pub user_first_name: String,
    pub user_last_name: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub role_public_id: PublicId,
    pub is_default_organization: bool,
    pub status: StaffStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StaffDetail {
    pub public_id: PublicId,
    pub user_public_id: PublicId,
    pub user_email: String,
    pub user_first_name: String,
    pub user_last_name: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub role_public_id: PublicId,
    pub branch_public_ids: Vec<PublicId>,
    pub is_default_organization: bool,
    pub status: StaffStatus,
}

pub struct StaffInvitationCreated {
    pub invitation_id: String,
    pub invitation_token: String,
    pub token_expires_at: chrono::DateTime<chrono::Utc>,
}

struct InvitationTokenBundle {
    token_id: String,
    token: String,
    token_hash: String,
    token_expires_at: chrono::DateTime<chrono::Utc>,
}

impl StaffService {
    pub async fn get_default_organization_for_user(
        db_transaction: &impl ConnectionTrait,
        user_id: PrimaryId,
    ) -> Result<Option<OrganizationModel>, AppError> {
        let staff = Staff::Entity::find()
            .filter(Staff::COLUMN.user_id.eq(user_id))
            .filter(Staff::COLUMN.status.eq(StaffStatus::Active))
            .filter(Staff::COLUMN.is_default_organization.eq(true))
            .one(db_transaction)
            .await?;

        match staff {
            Some(staff) => {
                let organization = Organization::Entity::find_by_id(staff.organization_id)
                    .one(db_transaction)
                    .await?
                    .ok_or(StaffServiceError::NotFound)?;
                Ok(Some(organization))
            }
            None => Ok(None),
        }
    }

    pub async fn get_organization_for_user(
        ctx: &ServiceContext,
        user_id: PrimaryId,
        organization_public_id: &str,
    ) -> Result<Organization::Model, AppError> {
        let organization = Organization::Entity::find()
            .filter(Organization::COLUMN.public_id.eq(organization_public_id))
            .one(&ctx.app_state.primary_read_replica)
            .await?;

        let Some(organization) = organization else {
            return Err(StaffServiceError::NotFound.into());
        };

        let staff = Staff::Entity::find()
            .filter(Staff::COLUMN.user_id.eq(user_id))
            .filter(Staff::COLUMN.organization_id.eq(organization.id))
            .filter(Staff::COLUMN.status.eq(StaffStatus::Active))
            .one(&ctx.app_state.primary_read_replica)
            .await?;

        staff
            .map(|_| organization)
            .ok_or(StaffServiceError::NotFound.into())
    }

    pub async fn create_staff_invitation(
        ctx: &ServiceContext,
        payload: CreateStaffInvitationInput,
    ) -> Result<StaffInvitationCreated, AppError> {
        let actor_id = ctx.get_actor_id()?;
        let organization_id = ctx.get_organization_id()?;
        let now = DateHelper::now().value();
        let invitation_public_id = IdGenerator::generate_general_id();
        let token_bundle = Self::build_invitation_token_bundle(ctx)?;
        let txn = ctx.app_state.primary_write_replica.begin().await?;

        let invitation = StaffInvitation::ActiveModel {
            public_id: Set(invitation_public_id.clone()),
            organization_id: Set(organization_id),
            invitee_email: Set(payload.invitee_email.trim().to_lowercase()),
            invitee_first_name: Set(payload.invitee_first_name),
            invitee_last_name: Set(payload.invitee_last_name),
            invited_role_id: Set(payload.invited_role_id),
            invitation_token_hash: Set(token_bundle.token_hash),
            invitation_token_id: Set(token_bundle.token_id),
            token_expires_at: Set(token_bundle.token_expires_at),
            accepted_at: Set(None),
            status: Set(StaffInvitationStatus::Pending),
            created_by_actor_id: Set(actor_id),
            updated_by_actor_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        Self::attach_invitation_to_branches(&txn, actor_id, invitation.id, &payload.branch_ids)
            .await?;
        txn.commit().await?;

        Ok(StaffInvitationCreated {
            invitation_id: invitation_public_id,
            invitation_token: token_bundle.token,
            token_expires_at: token_bundle.token_expires_at,
        })
    }

    pub async fn accept_staff_invitation(
        ctx: &ServiceContext,
        payload: AcceptStaffInvitationInput,
    ) -> Result<(), AppError> {
        let now = DateHelper::now().value();
        let invitation =
            Self::get_valid_invitation_from_token(ctx, payload.invitation_token.trim()).await?;
        Self::expire_invitation_if_needed(ctx, &invitation, now).await?;

        let settings = ctx.app_state.settings.clone();
        ctx.app_state
            .primary_write_replica
            .transaction::<_, (), AppError>(|txn| {
                let invitation = invitation.clone();
                let password = payload.password.clone();
                Box::pin(async move {
                    let user = Self::get_or_create_invited_user(txn, &invitation).await?;
                    Self::ensure_user_credential(txn, &settings, user.id, &password).await?;
                    let invitation_branch_ids =
                        Self::get_invitation_branch_ids(txn, invitation.id).await?;
                    let staff =
                        Self::find_or_create_staff_from_invitation(txn, &invitation, user.id, now)
                            .await?;
                    Self::attach_staff_to_branches(
                        txn,
                        invitation.created_by_actor_id,
                        staff.id,
                        &invitation_branch_ids,
                    )
                    .await?;
                    Self::mark_invitation_accepted(txn, &invitation, now).await?;
                    Self::revoke_other_pending_invitations(
                        txn,
                        invitation.id,
                        invitation.organization_id,
                        &invitation.invitee_email,
                        now,
                    )
                    .await?;

                    Ok(())
                })
            })
            .await?;

        Ok(())
    }

    pub async fn resend_staff_invitation(
        ctx: &ServiceContext,
        invitation_id: PrimaryId,
    ) -> Result<StaffInvitationCreated, AppError> {
        let actor_id = ctx.get_actor_id()?;
        let invitation =
            Self::staff_invitation_by_id(&ctx.app_state.primary_write_replica, invitation_id)
                .await?;

        if invitation.status != StaffInvitationStatus::Pending {
            return Err(StaffServiceError::InvitationAlreadyUsed.into());
        }

        let now = DateHelper::now().value();
        let token_bundle = Self::build_invitation_token_bundle(ctx)?;

        StaffInvitation::Entity::update_many()
            .col_expr(
                StaffInvitation::COLUMN.invitation_token_hash,
                sea_orm::sea_query::Expr::value(token_bundle.token_hash),
            )
            .col_expr(
                StaffInvitation::COLUMN.invitation_token_id,
                sea_orm::sea_query::Expr::value(token_bundle.token_id),
            )
            .col_expr(
                StaffInvitation::COLUMN.token_expires_at,
                sea_orm::sea_query::Expr::value(token_bundle.token_expires_at),
            )
            .col_expr(
                StaffInvitation::COLUMN.updated_by_actor_id,
                sea_orm::sea_query::Expr::value(Some(actor_id)),
            )
            .col_expr(
                StaffInvitation::COLUMN.updated_at,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(StaffInvitation::COLUMN.id.eq(invitation.id))
            .exec(&ctx.app_state.primary_write_replica)
            .await?;

        Ok(StaffInvitationCreated {
            invitation_id: invitation.public_id,
            invitation_token: token_bundle.token,
            token_expires_at: token_bundle.token_expires_at,
        })
    }

    pub async fn revoke_staff_invitation(
        ctx: &ServiceContext,
        invitation_id: PrimaryId,
    ) -> Result<(), AppError> {
        let actor_id = ctx.get_actor_id()?;
        let invitation =
            Self::staff_invitation_by_id(&ctx.app_state.primary_write_replica, invitation_id)
                .await?;

        if invitation.status != StaffInvitationStatus::Pending {
            return Err(StaffServiceError::InvitationAlreadyUsed.into());
        }

        let now = DateHelper::now().value();
        StaffInvitation::Entity::update_many()
            .col_expr(
                StaffInvitation::COLUMN.status,
                sea_orm::sea_query::Expr::value(StaffInvitationStatus::Revoked),
            )
            .col_expr(
                StaffInvitation::COLUMN.updated_by_actor_id,
                sea_orm::sea_query::Expr::value(Some(actor_id)),
            )
            .col_expr(
                StaffInvitation::COLUMN.updated_at,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(StaffInvitation::COLUMN.id.eq(invitation.id))
            .exec(&ctx.app_state.primary_write_replica)
            .await?;

        Ok(())
    }

    pub async fn list_staff_page(
        ctx: &ServiceContext,
        input: StaffListPageInput,
    ) -> Result<PageListResult<StaffListItem>, AppError> {
        let pagination = validate_page_pagination(input.page, input.per_page)?;
        let organization_id = ctx.get_organization_id()?;
        let sort_field = input.sort.unwrap_or(StaffSortField::CreatedAt);
        let sort_direction = input.direction.unwrap_or(SortDirection::Desc);

        let query =
            Self::build_staff_list_query(organization_id, input.name.as_deref(), input.status);
        let query = Self::apply_page_sort(query, sort_field, sort_direction);

        let result =
            execute_page_query(&ctx.app_state.primary_read_replica, query, pagination).await?;
        let rows = Self::map_staff_list_items(
            &ctx.app_state.primary_read_replica,
            organization_id,
            result.rows,
        )
        .await?;

        Ok(PageListResult {
            rows,
            meta: result.meta,
        })
    }

    pub async fn get_staff(ctx: &ServiceContext, public_id: &str) -> Result<StaffDetail, AppError> {
        let organization_id = ctx.get_organization_id()?;
        let staff = Self::find_staff_by_public_id(
            &ctx.app_state.primary_read_replica,
            organization_id,
            public_id,
        )
        .await?;
        let user = User::Entity::find_by_id(staff.user_id)
            .one(&ctx.app_state.primary_read_replica)
            .await?
            .ok_or_else(|| AppError::InternalServer("Staff user not found".into()))?;
        let role = StaffRole::Entity::find_by_id(staff.role_id)
            .one(&ctx.app_state.primary_read_replica)
            .await?
            .ok_or_else(|| AppError::InternalServer("Staff role not found".into()))?;
        let branch_public_ids =
            Self::branch_public_ids_for_staff(ctx, organization_id, staff.id).await?;

        Ok(StaffDetail {
            public_id: staff.public_id,
            user_public_id: user.public_id,
            user_email: user.email,
            user_first_name: user.first_name,
            user_last_name: user.last_name,
            name_primary: staff.name_primary,
            name_secondary: staff.name_secondary,
            role_public_id: role.public_id,
            branch_public_ids,
            is_default_organization: staff.is_default_organization,
            status: staff.status,
        })
    }

    pub async fn update_staff(
        ctx: &ServiceContext,
        public_id: &str,
        payload: UpdateStaffInput,
    ) -> Result<StaffDetail, AppError> {
        let actor_id = ctx.get_actor_id()?;
        let organization_id = ctx.get_organization_id()?;
        let now = DateHelper::now().value();
        let existing = Self::find_staff_by_public_id(
            &ctx.app_state.primary_write_replica,
            organization_id,
            public_id,
        )
        .await?;
        let branch_ids = payload.branch_ids.clone();

        let txn = ctx.app_state.primary_write_replica.begin().await?;
        if payload.is_default_organization == Some(true) {
            Self::clear_other_default_organizations(
                &txn,
                actor_id,
                existing.user_id,
                existing.id,
                now,
            )
            .await?;
        }

        let mut updated = existing.into_active_model();
        if let Some(name_primary) = payload.name_primary {
            updated.name_primary = Set(name_primary);
        }
        if payload.name_secondary.is_some() {
            updated.name_secondary = Set(payload.name_secondary);
        }
        if let Some(role_id) = payload.role_id {
            updated.role_id = Set(role_id);
        }
        if let Some(is_default_organization) = payload.is_default_organization {
            updated.is_default_organization = Set(is_default_organization);
        }
        if let Some(status) = payload.status {
            updated.status = Set(status);
        }
        updated.updated_by_actor_id = Set(Some(actor_id));
        updated.updated_at = Set(now);

        let staff = updated.update(&txn).await?;
        if let Some(branch_ids) = branch_ids {
            Self::replace_staff_branches(&txn, actor_id, staff.id, &branch_ids).await?;
        }
        txn.commit().await?;

        Self::get_staff(ctx, &staff.public_id).await
    }

    pub async fn delete_staff(ctx: &ServiceContext, public_id: &str) -> Result<(), AppError> {
        let actor_id = ctx.get_actor_id()?;
        let organization_id = ctx.get_organization_id()?;
        let now = DateHelper::now().value();
        let staff = Self::find_staff_by_public_id(
            &ctx.app_state.primary_read_replica,
            organization_id,
            public_id,
        )
        .await?;
        let txn = ctx.app_state.primary_write_replica.begin().await?;

        let result = Staff::Entity::update_many()
            .col_expr(
                Staff::Column::Status,
                sea_orm::sea_query::Expr::value(StaffStatus::Deleted),
            )
            .col_expr(
                Staff::Column::UpdatedByActorId,
                sea_orm::sea_query::Expr::value(Some(actor_id)),
            )
            .col_expr(
                Staff::Column::UpdatedAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(Staff::Column::Id.eq(staff.id))
            .filter(Staff::Column::Status.ne(StaffStatus::Deleted))
            .exec(&txn)
            .await?;

        if result.rows_affected == 0 {
            return Err(StaffServiceError::NotFound.into());
        }

        StaffBranch::Entity::update_many()
            .col_expr(
                StaffBranch::Column::Status,
                sea_orm::sea_query::Expr::value(GenericStatus::Deleted),
            )
            .col_expr(
                StaffBranch::Column::UpdatedByActorId,
                sea_orm::sea_query::Expr::value(Some(actor_id)),
            )
            .col_expr(
                StaffBranch::Column::UpdatedAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(StaffBranch::Column::StaffId.eq(staff.id))
            .filter(StaffBranch::Column::Status.ne(GenericStatus::Deleted))
            .exec(&txn)
            .await?;

        txn.commit().await?;

        Ok(())
    }

    pub async fn create_staff_from_user(
        db_transaction: &impl ConnectionTrait,
        ctx: &ServiceContext,
        organization_id: PrimaryId,
        branch_ids: &[PrimaryId],
        role_id: PrimaryId,
    ) -> Result<(), AppError> {
        let actor_id = ctx.get_actor_id()?;
        let user_id = ctx.get_user_id()?;
        let user = UserService::get_user_by_id(&ctx, user_id).await?;

        let now = DateHelper::now().value();
        let public_id = IdGenerator::generate_general_id();
        let is_default_organization =
            Self::get_default_organization_for_user(db_transaction, user_id)
                .await?
                .is_none();

        let staff = Staff::ActiveModel {
            user_id: Set(user_id),
            organization_id: Set(organization_id),
            public_id: Set(public_id),
            name_primary: Set(format!("{} {}", user.first_name, user.last_name)),
            name_secondary: Set(None),
            role_id: Set(role_id),
            is_default_organization: Set(is_default_organization),
            status: Set(StaffStatus::Active),
            created_by_actor_id: Set(actor_id),
            updated_by_actor_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(db_transaction)
        .await?;

        Self::attach_staff_to_branches(db_transaction, actor_id, staff.id, branch_ids).await?;

        Ok(())
    }

    async fn attach_invitation_to_branches(
        db_transaction: &impl ConnectionTrait,
        actor_id: PrimaryId,
        invitation_id: PrimaryId,
        branch_ids: &[PrimaryId],
    ) -> Result<(), AppError> {
        let now = DateHelper::now().value();
        for branch_id in branch_ids.iter().copied() {
            let exists = StaffInvitationBranch::Entity::find()
                .filter(StaffInvitationBranch::Column::StaffInvitationId.eq(invitation_id))
                .filter(StaffInvitationBranch::Column::BranchId.eq(branch_id))
                .one(db_transaction)
                .await?
                .is_some();

            if exists {
                continue;
            }

            StaffInvitationBranch::ActiveModel {
                status: Set(GenericStatus::Active),
                staff_invitation_id: Set(invitation_id),
                branch_id: Set(branch_id),
                created_by_actor_id: Set(actor_id),
                updated_by_actor_id: Set(None),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            }
            .insert(db_transaction)
            .await?;
        }

        Ok(())
    }

    async fn attach_staff_to_branches(
        db_transaction: &impl ConnectionTrait,
        actor_id: PrimaryId,
        staff_id: PrimaryId,
        branch_ids: &[PrimaryId],
    ) -> Result<(), AppError> {
        let now = DateHelper::now().value();
        for branch_id in branch_ids.iter().copied() {
            let exists = StaffBranch::Entity::find()
                .filter(StaffBranch::Column::StaffId.eq(staff_id))
                .filter(StaffBranch::Column::BranchId.eq(branch_id))
                .one(db_transaction)
                .await?;

            if let Some(existing) = exists {
                if existing.status != GenericStatus::Active {
                    StaffBranch::Entity::update_many()
                        .col_expr(
                            StaffBranch::Column::Status,
                            sea_orm::sea_query::Expr::value(GenericStatus::Active),
                        )
                        .col_expr(
                            StaffBranch::Column::UpdatedByActorId,
                            sea_orm::sea_query::Expr::value(Some(actor_id)),
                        )
                        .col_expr(
                            StaffBranch::Column::UpdatedAt,
                            sea_orm::sea_query::Expr::value(now),
                        )
                        .filter(StaffBranch::Column::Id.eq(existing.id))
                        .exec(db_transaction)
                        .await?;
                }
                continue;
            }

            StaffBranch::ActiveModel {
                status: Set(GenericStatus::Active),
                staff_id: Set(staff_id),
                branch_id: Set(branch_id),
                created_by_actor_id: Set(actor_id),
                updated_by_actor_id: Set(None),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            }
            .insert(db_transaction)
            .await?;
        }

        Ok(())
    }

    async fn replace_staff_branches(
        db_transaction: &impl ConnectionTrait,
        actor_id: PrimaryId,
        staff_id: PrimaryId,
        branch_ids: &[PrimaryId],
    ) -> Result<(), AppError> {
        let now = DateHelper::now().value();
        let desired_branch_ids = branch_ids.iter().copied().collect::<HashSet<_>>();

        StaffBranch::Entity::update_many()
            .col_expr(
                StaffBranch::Column::Status,
                sea_orm::sea_query::Expr::value(GenericStatus::Deleted),
            )
            .col_expr(
                StaffBranch::Column::UpdatedByActorId,
                sea_orm::sea_query::Expr::value(Some(actor_id)),
            )
            .col_expr(
                StaffBranch::Column::UpdatedAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(StaffBranch::Column::StaffId.eq(staff_id))
            .filter(StaffBranch::Column::Status.eq(GenericStatus::Active))
            .filter(StaffBranch::Column::BranchId.is_not_in(desired_branch_ids))
            .exec(db_transaction)
            .await?;

        Self::attach_staff_to_branches(db_transaction, actor_id, staff_id, branch_ids).await
    }

    async fn clear_other_default_organizations(
        db_transaction: &impl ConnectionTrait,
        actor_id: PrimaryId,
        user_id: PrimaryId,
        staff_id: PrimaryId,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), AppError> {
        Staff::Entity::update_many()
            .col_expr(
                Staff::Column::IsDefaultOrganization,
                sea_orm::sea_query::Expr::value(false),
            )
            .col_expr(
                Staff::Column::UpdatedByActorId,
                sea_orm::sea_query::Expr::value(Some(actor_id)),
            )
            .col_expr(
                Staff::Column::UpdatedAt,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(Staff::Column::UserId.eq(user_id))
            .filter(Staff::Column::Id.ne(staff_id))
            .filter(Staff::Column::Status.ne(StaffStatus::Deleted))
            .exec(db_transaction)
            .await?;

        Ok(())
    }

    fn build_invitation_token_bundle(
        ctx: &ServiceContext,
    ) -> Result<InvitationTokenBundle, AppError> {
        let now = DateHelper::now().value();
        let token_expires_at = DateHelper::from(now).add_days(2).value();
        let token_id = IdGenerator::generate_general_id();
        let token_secret = format!(
            "{}.{}",
            IdGenerator::generate_general_id(),
            IdGenerator::generate_general_id()
        );
        let token = format!("{}.{}", token_id, token_secret);
        let token_hash = PasswordHelpers::hash_secret(&ctx.app_state.settings, &token_secret)?;

        Ok(InvitationTokenBundle {
            token_id,
            token,
            token_hash,
            token_expires_at,
        })
    }

    async fn get_valid_invitation_from_token(
        ctx: &ServiceContext,
        invitation_token: &str,
    ) -> Result<StaffInvitation::Model, AppError> {
        let (invitation_token_id, invitation_token_secret) = invitation_token
            .split_once('.')
            .ok_or(StaffServiceError::InvitationNotFound)?;
        let invitation = StaffInvitation::Entity::find()
            .filter(
                StaffInvitation::COLUMN
                    .invitation_token_id
                    .eq(invitation_token_id),
            )
            .one(&ctx.app_state.primary_read_replica)
            .await?
            .ok_or(StaffServiceError::InvitationNotFound)?;

        let is_valid_token = PasswordHelpers::verify_secret(
            &ctx.app_state.settings,
            invitation_token_secret,
            &invitation.invitation_token_hash,
        )?;
        if !is_valid_token {
            return Err(StaffServiceError::InvitationNotFound.into());
        }

        if invitation.status != StaffInvitationStatus::Pending {
            return Err(StaffServiceError::InvitationAlreadyUsed.into());
        }

        Ok(invitation)
    }

    async fn expire_invitation_if_needed(
        ctx: &ServiceContext,
        invitation: &StaffInvitation::Model,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), AppError> {
        if invitation.token_expires_at >= Utc::now() {
            return Ok(());
        }

        StaffInvitation::Entity::update_many()
            .col_expr(
                StaffInvitation::COLUMN.status,
                sea_orm::sea_query::Expr::value(StaffInvitationStatus::Expired),
            )
            .col_expr(
                StaffInvitation::COLUMN.updated_at,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(StaffInvitation::COLUMN.id.eq(invitation.id))
            .exec(&ctx.app_state.primary_write_replica)
            .await?;

        Err(StaffServiceError::InvitationExpired.into())
    }

    async fn get_or_create_invited_user(
        db_transaction: &impl ConnectionTrait,
        invitation: &StaffInvitation::Model,
    ) -> Result<UserModel, AppError> {
        UserService::get_or_create_user_without_password(
            db_transaction,
            invitation.invitee_first_name.clone(),
            invitation.invitee_last_name.clone(),
            invitation.invitee_email.clone(),
        )
        .await
    }

    async fn ensure_user_credential(
        db_transaction: &DatabaseTransaction,
        settings: &Settings,
        user_id: PrimaryId,
        password: &String,
    ) -> Result<(), AppError> {
        let has_credential =
            UserCredentialService::credential_exists(db_transaction, user_id).await?;
        if !has_credential {
            UserCredentialService::save_credential(settings, db_transaction, user_id, password)
                .await?;
        }
        Ok(())
    }

    async fn get_invitation_branch_ids(
        db_transaction: &impl ConnectionTrait,
        invitation_id: PrimaryId,
    ) -> Result<Vec<PrimaryId>, AppError> {
        Ok(StaffInvitationBranch::Entity::find()
            .filter(StaffInvitationBranch::Column::StaffInvitationId.eq(invitation_id))
            .all(db_transaction)
            .await?
            .into_iter()
            .map(|item| item.branch_id)
            .collect())
    }

    async fn find_or_create_staff_from_invitation(
        db_transaction: &impl ConnectionTrait,
        invitation: &StaffInvitation::Model,
        user_id: PrimaryId,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<Staff::Model, AppError> {
        if let Some(staff) = Staff::Entity::find()
            .filter(Staff::COLUMN.user_id.eq(user_id))
            .filter(Staff::COLUMN.organization_id.eq(invitation.organization_id))
            .one(db_transaction)
            .await?
        {
            return Ok(staff);
        }

        let is_default_organization =
            Self::get_default_organization_for_user(db_transaction, user_id)
                .await?
                .is_none();
        Staff::ActiveModel {
            user_id: Set(user_id),
            organization_id: Set(invitation.organization_id),
            public_id: Set(IdGenerator::generate_general_id()),
            name_primary: Set(format!(
                "{} {}",
                invitation.invitee_first_name, invitation.invitee_last_name
            )),
            name_secondary: Set(None),
            role_id: Set(invitation.invited_role_id),
            is_default_organization: Set(is_default_organization),
            status: Set(StaffStatus::Active),
            created_by_actor_id: Set(invitation.created_by_actor_id),
            updated_by_actor_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(db_transaction)
        .await
        .map_err(Into::into)
    }

    async fn mark_invitation_accepted(
        db_transaction: &impl ConnectionTrait,
        invitation: &StaffInvitation::Model,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), AppError> {
        StaffInvitation::Entity::update_many()
            .col_expr(
                StaffInvitation::COLUMN.status,
                sea_orm::sea_query::Expr::value(StaffInvitationStatus::Accepted),
            )
            .col_expr(
                StaffInvitation::COLUMN.accepted_at,
                sea_orm::sea_query::Expr::value(Some(now)),
            )
            .col_expr(
                StaffInvitation::COLUMN.updated_at,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(StaffInvitation::COLUMN.id.eq(invitation.id))
            .exec(db_transaction)
            .await?;
        Ok(())
    }

    async fn revoke_other_pending_invitations(
        db_transaction: &impl ConnectionTrait,
        accepted_invitation_id: PrimaryId,
        organization_id: PrimaryId,
        invitee_email: &str,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), AppError> {
        StaffInvitation::Entity::update_many()
            .col_expr(
                StaffInvitation::COLUMN.status,
                sea_orm::sea_query::Expr::value(StaffInvitationStatus::Revoked),
            )
            .col_expr(
                StaffInvitation::COLUMN.updated_at,
                sea_orm::sea_query::Expr::value(now),
            )
            .filter(StaffInvitation::COLUMN.id.ne(accepted_invitation_id))
            .filter(StaffInvitation::COLUMN.organization_id.eq(organization_id))
            .filter(
                StaffInvitation::COLUMN
                    .invitee_email
                    .eq(invitee_email.to_string()),
            )
            .filter(
                StaffInvitation::COLUMN
                    .status
                    .eq(StaffInvitationStatus::Pending),
            )
            .exec(db_transaction)
            .await?;
        Ok(())
    }

    async fn staff_invitation_by_id(
        db_transaction: &impl ConnectionTrait,
        invitation_id: PrimaryId,
    ) -> Result<StaffInvitation::Model, AppError> {
        StaffInvitation::Entity::find_by_id(invitation_id)
            .one(db_transaction)
            .await?
            .ok_or(StaffServiceError::InvitationNotFound.into())
    }

    async fn find_staff_by_public_id(
        db: &impl ConnectionTrait,
        organization_id: PrimaryId,
        public_id: &str,
    ) -> Result<Staff::Model, AppError> {
        Staff::Entity::find()
            .filter(Staff::Column::OrganizationId.eq(organization_id))
            .filter(Staff::Column::PublicId.eq(public_id))
            .filter(Staff::Column::Status.ne(StaffStatus::Deleted))
            .one(db)
            .await?
            .ok_or_else(|| StaffServiceError::NotFound.into())
    }

    fn build_staff_list_query(
        organization_id: PrimaryId,
        name: Option<&str>,
        status: Option<StaffStatus>,
    ) -> sea_orm::Select<Staff::Entity> {
        let mut query =
            Staff::Entity::find().filter(Staff::Column::OrganizationId.eq(organization_id));

        if let Some(name) = trim_and_filter_empty(name) {
            query = query.filter(Staff::Column::NamePrimary.contains(name));
        }

        if let Some(status) = status {
            query = query.filter(Staff::Column::Status.eq(status));
        } else {
            query = query.filter(Staff::Column::Status.ne(StaffStatus::Deleted));
        }

        query
    }

    fn apply_page_sort(
        query: sea_orm::Select<Staff::Entity>,
        sort_field: StaffSortField,
        sort_direction: SortDirection,
    ) -> sea_orm::Select<Staff::Entity> {
        match (sort_field, sort_direction) {
            (StaffSortField::CreatedAt, SortDirection::Asc) => query
                .order_by_asc(Staff::Column::CreatedAt)
                .order_by_asc(Staff::Column::Id),
            (StaffSortField::CreatedAt, SortDirection::Desc) => query
                .order_by_desc(Staff::Column::CreatedAt)
                .order_by_desc(Staff::Column::Id),
            (StaffSortField::NamePrimary, SortDirection::Asc) => query
                .order_by_asc(Staff::Column::NamePrimary)
                .order_by_asc(Staff::Column::Id),
            (StaffSortField::NamePrimary, SortDirection::Desc) => query
                .order_by_desc(Staff::Column::NamePrimary)
                .order_by_desc(Staff::Column::Id),
        }
    }

    async fn map_staff_list_items(
        db: &impl ConnectionTrait,
        organization_id: PrimaryId,
        staffs: Vec<Staff::Model>,
    ) -> Result<Vec<StaffListItem>, AppError> {
        if staffs.is_empty() {
            return Ok(Vec::new());
        }

        let user_ids = staffs
            .iter()
            .map(|staff| staff.user_id)
            .collect::<HashSet<_>>();
        let role_ids = staffs
            .iter()
            .map(|staff| staff.role_id)
            .collect::<HashSet<_>>();

        let users = User::Entity::find()
            .filter(User::Column::Id.is_in(user_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|user| (user.id, user))
            .collect::<HashMap<_, _>>();
        let roles = StaffRole::Entity::find()
            .filter(StaffRole::Column::OrganizationId.eq(organization_id))
            .filter(StaffRole::Column::Id.is_in(role_ids))
            .all(db)
            .await?
            .into_iter()
            .map(|role| (role.id, role))
            .collect::<HashMap<_, _>>();

        staffs
            .into_iter()
            .map(|staff| {
                let user = users
                    .get(&staff.user_id)
                    .ok_or_else(|| AppError::InternalServer("Staff user not found".into()))?;
                let role = roles
                    .get(&staff.role_id)
                    .ok_or_else(|| AppError::InternalServer("Staff role not found".into()))?;

                Ok(StaffListItem {
                    public_id: staff.public_id,
                    user_public_id: user.public_id.clone(),
                    user_email: user.email.clone(),
                    user_first_name: user.first_name.clone(),
                    user_last_name: user.last_name.clone(),
                    name_primary: staff.name_primary,
                    name_secondary: staff.name_secondary,
                    role_public_id: role.public_id.clone(),
                    is_default_organization: staff.is_default_organization,
                    status: staff.status,
                })
            })
            .collect()
    }

    async fn branch_public_ids_for_staff(
        ctx: &ServiceContext,
        organization_id: PrimaryId,
        staff_id: PrimaryId,
    ) -> Result<Vec<PublicId>, AppError> {
        let staff_branches = StaffBranch::Entity::find()
            .filter(StaffBranch::Column::StaffId.eq(staff_id))
            .filter(StaffBranch::Column::Status.eq(GenericStatus::Active))
            .order_by_asc(StaffBranch::Column::Id)
            .all(&ctx.app_state.primary_read_replica)
            .await?;
        let branch_ids = staff_branches
            .iter()
            .map(|staff_branch| staff_branch.branch_id)
            .collect::<Vec<_>>();

        if branch_ids.is_empty() {
            return Ok(Vec::new());
        }

        let branches = Branch::Entity::find()
            .filter(Branch::Column::OrganizationId.eq(organization_id))
            .filter(Branch::Column::Id.is_in(branch_ids.iter().copied()))
            .filter(Branch::Column::Status.ne(BranchStatus::Deleted))
            .all(&ctx.app_state.primary_read_replica)
            .await?
            .into_iter()
            .map(|branch| (branch.id, branch.public_id))
            .collect::<HashMap<_, _>>();

        branch_ids
            .into_iter()
            .map(|branch_id| {
                branches
                    .get(&branch_id)
                    .cloned()
                    .ok_or_else(|| AppError::InternalServer("Staff branch not found".into()))
            })
            .collect()
    }
}
