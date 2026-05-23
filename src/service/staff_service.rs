use crate::entity::organization::organization_entity as Organization;
use crate::entity::organization::organization_entity::OrganizationModel;
use crate::entity::staff::staff_branch_entity as StaffBranch;
use crate::entity::staff::staff_entity::{self as Staff, StaffStatus};
use crate::entity::staff::staff_invitation_branch_entity as StaffInvitationBranch;
use crate::entity::staff::staff_invitation_entity::{
    self as StaffInvitation, StaffInvitationStatus,
};
use crate::entity::{BranchPrimaryId, OrganizationPrimaryId, StaffPrimaryId, UserPrimaryId};
use crate::errors::app_error::AppError;
use crate::errors::staff_service_errors::StaffServiceError;
use crate::resolver::staff_payload_resolver::ResolvedCreateStaffInvitation;
use crate::service::service_context::ServiceContext;
use crate::service::user_credential_service::UserCredentialService;
use crate::service::user_service::UserService;
use crate::utils::date_helpers::DateHelper;
use crate::utils::id_generator::IdGenerator;
use crate::utils::password_helpers::PasswordHelpers;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseTransaction, EntityTrait, QueryFilter,
    Set, TransactionTrait,
};

pub struct StaffService;

pub struct CreateStaffInvitationInput {
    pub invitee_email: String,
    pub invitee_first_name: String,
    pub invitee_last_name: String,
    pub role_public_id: String,
    pub branch_public_ids: Option<Vec<String>>,
}

pub struct AcceptStaffInvitationInput {
    pub invitation_token: String,
    pub password: String,
}

pub struct ResendStaffInvitationInput {
    pub invitation_id: String,
}

pub struct RevokeStaffInvitationInput {
    pub invitation_id: String,
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
        user_id: UserPrimaryId,
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
        user_id: UserPrimaryId,
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
        payload: ResolvedCreateStaffInvitation,
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
        invitation: StaffInvitation::Model,
    ) -> Result<StaffInvitationCreated, AppError> {
        let actor_id = ctx.get_actor_id()?;

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
        invitation: StaffInvitation::Model,
    ) -> Result<(), AppError> {
        let actor_id = ctx.get_actor_id()?;

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

    pub async fn create_staff_from_user(
        db_transaction: &impl ConnectionTrait,
        ctx: &ServiceContext,
        organization_id: OrganizationPrimaryId,
        branch_ids: &[BranchPrimaryId],
        role_id: i32,
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
        actor_id: i32,
        invitation_id: i32,
        branch_ids: &[BranchPrimaryId],
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
        actor_id: i32,
        staff_id: StaffPrimaryId,
        branch_ids: &[BranchPrimaryId],
    ) -> Result<(), AppError> {
        let now = DateHelper::now().value();
        for branch_id in branch_ids.iter().copied() {
            let exists = StaffBranch::Entity::find()
                .filter(StaffBranch::Column::StaffId.eq(staff_id))
                .filter(StaffBranch::Column::BranchId.eq(branch_id))
                .one(db_transaction)
                .await?
                .is_some();

            if exists {
                continue;
            }

            StaffBranch::ActiveModel {
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
    ) -> Result<crate::entity::user_entity::Model, AppError> {
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
        settings: &crate::config::settings::Settings,
        user_id: UserPrimaryId,
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
        invitation_id: i32,
    ) -> Result<Vec<BranchPrimaryId>, AppError> {
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
        user_id: UserPrimaryId,
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
        accepted_invitation_id: i32,
        organization_id: OrganizationPrimaryId,
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
}
