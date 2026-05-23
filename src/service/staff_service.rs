use crate::entity::organization::organization_entity as Organization;
use crate::entity::{OrganizationPrimaryId, UserPrimaryId};
use crate::errors::app_error::AppError;
use crate::errors::staff_service_errors::StaffServiceError;
use crate::service::organization_service::OrganizationService;
use crate::service::service_context::ServiceContext;
use crate::service::user_credential_service::UserCredentialService;
use crate::service::user_service::UserService;
use crate::utils::password_helpers::PasswordHelpers;
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set, TransactionTrait,
};
use serde::Deserialize;

use crate::entity::organization::staff_entity::{self as Staff, StaffStatus};
use crate::entity::organization::staff_invitation_entity::{
    self as StaffInvitation, StaffInvitationStatus,
};
use crate::utils::date_helpers::DateHelper;
use crate::utils::id_generator::IdGenerator;

pub struct StaffService;

#[derive(Deserialize)]
pub struct CreateStaffInvitation {
    pub organization_public_id: String,
    pub invitee_email: String,
    pub invitee_first_name: String,
    pub invitee_last_name: String,
    pub invited_role: Option<String>,
}

#[derive(Deserialize)]
pub struct AcceptStaffInvitation {
    pub invitation_token: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct ResendStaffInvitation {
    pub invitation_id: String,
}

#[derive(Deserialize)]
pub struct RevokeStaffInvitation {
    pub invitation_id: String,
}

pub struct StaffInvitationCreated {
    pub invitation_id: String,
    pub invitation_token: String,
    pub token_expires_at: chrono::DateTime<chrono::Utc>,
}

impl StaffService {
    pub async fn get_default_organization_for_user(
        ctx: &ServiceContext,
        user_id: UserPrimaryId,
    ) -> Result<Option<Organization::Model>, AppError> {
        let staff = Staff::Entity::find()
            .filter(Staff::COLUMN.user_id.eq(user_id))
            .filter(Staff::COLUMN.status.eq(StaffStatus::Active))
            .filter(Staff::COLUMN.is_default_organization.eq(true))
            .one(&ctx.app_state.primary_read_replica)
            .await?;

        match staff {
            Some(staff) => {
                let organization = Organization::Entity::find_by_id(staff.organization_id)
                    .one(&ctx.app_state.primary_read_replica)
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
        payload: CreateStaffInvitation,
    ) -> Result<StaffInvitationCreated, AppError> {
        let actor_id = ctx.get_actor_id()?;
        let organization = OrganizationService::get_organization_by_public_id(
            ctx,
            &payload.organization_public_id,
        )
        .await?;

        let now = DateHelper::now().value();
        let token_expires_at = DateHelper::from(now).add_days(2).value();
        let invitation_public_id = IdGenerator::generate_general_id();
        let invitation_token_id = IdGenerator::generate_general_id();
        let invitation_token_secret = format!(
            "{}.{}",
            IdGenerator::generate_general_id(),
            IdGenerator::generate_general_id()
        );
        let invitation_token = format!("{}.{}", invitation_token_id, invitation_token_secret);
        let invitation_token_hash =
            PasswordHelpers::hash_secret(&ctx.app_state.settings, &invitation_token_secret)?;

        let invitation = StaffInvitation::ActiveModel {
            public_id: Set(invitation_public_id.clone()),
            organization_id: Set(organization.id),
            invitee_email: Set(payload.invitee_email.trim().to_lowercase()),
            invitee_first_name: Set(payload.invitee_first_name),
            invitee_last_name: Set(payload.invitee_last_name),
            invited_role: Set(payload.invited_role),
            invitation_token_hash: Set(invitation_token_hash),
            invitation_token_id: Set(invitation_token_id),
            token_expires_at: Set(token_expires_at),
            accepted_at: Set(None),
            status: Set(StaffInvitationStatus::Pending),
            created_by_actor_id: Set(actor_id),
            updated_by_actor_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        invitation
            .insert(&ctx.app_state.primary_write_replica)
            .await?;

        Ok(StaffInvitationCreated {
            invitation_id: invitation_public_id,
            invitation_token,
            token_expires_at,
        })
    }

    pub async fn accept_staff_invitation(
        ctx: &ServiceContext,
        payload: AcceptStaffInvitation,
    ) -> Result<(), AppError> {
        let now = DateHelper::now().value();
        let invitation_token = payload.invitation_token.trim();
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
        if invitation.token_expires_at < Utc::now() {
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
            return Err(StaffServiceError::InvitationExpired.into());
        }

        let settings = ctx.app_state.settings.clone();
        ctx.app_state
            .primary_write_replica
            .transaction::<_, (), AppError>(|txn| {
                let invitation = invitation.clone();
                let password = payload.password.clone();
                Box::pin(async move {
                    let user = UserService::get_or_create_user_without_password(
                        txn,
                        invitation.invitee_first_name.clone(),
                        invitation.invitee_last_name.clone(),
                        invitation.invitee_email.clone(),
                    )
                    .await?;

                    let has_credential =
                        UserCredentialService::credential_exists(txn, user.id).await?;
                    if !has_credential {
                        UserCredentialService::save_credential(&settings, txn, user.id, &password)
                            .await?;
                    }

                    let already_staff = Staff::Entity::find()
                        .filter(Staff::COLUMN.user_id.eq(user.id))
                        .filter(Staff::COLUMN.organization_id.eq(invitation.organization_id))
                        .one(txn)
                        .await?
                        .is_some();

                    if !already_staff {
                        let is_default_organization =
                            !Self::user_has_default_organization(txn, user.id).await?;
                        let staff = Staff::ActiveModel {
                            user_id: Set(user.id),
                            organization_id: Set(invitation.organization_id),
                            public_id: Set(IdGenerator::generate_general_id()),
                            name_primary: Set(format!(
                                "{} {}",
                                invitation.invitee_first_name, invitation.invitee_last_name
                            )),
                            name_secondary: Set(None),
                            is_default_organization: Set(is_default_organization),
                            status: Set(StaffStatus::Active),
                            created_by_actor_id: Set(invitation.created_by_actor_id),
                            updated_by_actor_id: Set(None),
                            created_at: Set(now),
                            updated_at: Set(now),
                            ..Default::default()
                        };
                        staff.insert(txn).await?;
                    }

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
                        .exec(txn)
                        .await?;

                    StaffInvitation::Entity::update_many()
                        .col_expr(
                            StaffInvitation::COLUMN.status,
                            sea_orm::sea_query::Expr::value(StaffInvitationStatus::Revoked),
                        )
                        .col_expr(
                            StaffInvitation::COLUMN.updated_at,
                            sea_orm::sea_query::Expr::value(now),
                        )
                        .filter(StaffInvitation::COLUMN.id.ne(invitation.id))
                        .filter(
                            StaffInvitation::COLUMN
                                .organization_id
                                .eq(invitation.organization_id),
                        )
                        .filter(
                            StaffInvitation::COLUMN
                                .invitee_email
                                .eq(invitation.invitee_email.clone()),
                        )
                        .filter(
                            StaffInvitation::COLUMN
                                .status
                                .eq(StaffInvitationStatus::Pending),
                        )
                        .exec(txn)
                        .await?;

                    Ok(())
                })
            })
            .await?;

        Ok(())
    }

    pub async fn resend_staff_invitation(
        ctx: &ServiceContext,
        payload: ResendStaffInvitation,
    ) -> Result<StaffInvitationCreated, AppError> {
        let actor_id = ctx.get_actor_id()?;
        let invitation = StaffInvitation::Entity::find()
            .filter(
                StaffInvitation::COLUMN
                    .public_id
                    .eq(payload.invitation_id.clone()),
            )
            .one(&ctx.app_state.primary_read_replica)
            .await?
            .ok_or(StaffServiceError::InvitationNotFound)?;

        if invitation.status != StaffInvitationStatus::Pending {
            return Err(StaffServiceError::InvitationAlreadyUsed.into());
        }

        let now = DateHelper::now().value();
        let token_expires_at = DateHelper::from(now).add_days(2).value();
        let invitation_token_id = IdGenerator::generate_general_id();
        let invitation_token_secret = format!(
            "{}.{}",
            IdGenerator::generate_general_id(),
            IdGenerator::generate_general_id()
        );
        let invitation_token = format!("{}.{}", invitation_token_id, invitation_token_secret);
        let invitation_token_hash =
            PasswordHelpers::hash_secret(&ctx.app_state.settings, &invitation_token_secret)?;

        StaffInvitation::Entity::update_many()
            .col_expr(
                StaffInvitation::COLUMN.invitation_token_hash,
                sea_orm::sea_query::Expr::value(invitation_token_hash),
            )
            .col_expr(
                StaffInvitation::COLUMN.invitation_token_id,
                sea_orm::sea_query::Expr::value(invitation_token_id),
            )
            .col_expr(
                StaffInvitation::COLUMN.token_expires_at,
                sea_orm::sea_query::Expr::value(token_expires_at),
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
            invitation_token,
            token_expires_at,
        })
    }

    pub async fn revoke_staff_invitation(
        ctx: &ServiceContext,
        payload: RevokeStaffInvitation,
    ) -> Result<(), AppError> {
        let actor_id = ctx.get_actor_id()?;
        let invitation = StaffInvitation::Entity::find()
            .filter(StaffInvitation::COLUMN.public_id.eq(payload.invitation_id))
            .one(&ctx.app_state.primary_read_replica)
            .await?
            .ok_or(StaffServiceError::InvitationNotFound)?;

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
    ) -> Result<(), AppError> {
        let actor_id = ctx.get_actor_id()?;
        let user_id = ctx.get_user_id()?;
        let user = UserService::get_user_by_id(&ctx, user_id).await?;

        let now = DateHelper::now().value();
        let public_id = IdGenerator::generate_general_id();
        let is_default_organization =
            !Self::user_has_default_organization(db_transaction, user_id).await?;

        let staff_active_model = Staff::ActiveModel {
            user_id: Set(user_id),
            organization_id: Set(organization_id),
            public_id: Set(public_id),
            name_primary: Set(format!("{} {}", user.first_name, user.last_name)),
            name_secondary: Set(None),
            is_default_organization: Set(is_default_organization),
            status: Set(StaffStatus::Active),
            created_by_actor_id: Set(actor_id),
            updated_by_actor_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        staff_active_model.insert(db_transaction).await?;

        Ok(())
    }

    async fn user_has_default_organization(
        db_transaction: &impl ConnectionTrait,
        user_id: UserPrimaryId,
    ) -> Result<bool, AppError> {
        Ok(Staff::Entity::find()
            .filter(Staff::COLUMN.user_id.eq(user_id))
            .filter(Staff::COLUMN.status.eq(StaffStatus::Active))
            .filter(Staff::COLUMN.is_default_organization.eq(true))
            .one(db_transaction)
            .await?
            .is_some())
    }
}
