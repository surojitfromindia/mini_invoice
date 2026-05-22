use crate::entity::login_log_entity::SignInLogEventType;
use crate::errors::app_error::AppError;
use crate::errors::jwt_errors::JwtError;
use crate::errors::user_service_errors::UserServiceError;
use crate::service::login_log_service::LoginLogsService;
use crate::service::service_context::ServiceContext;
use crate::service::user_credential_service::UserCredentialService;
use crate::service::user_service::UserService;
use crate::utils::jwt_helpers::JwtHelpers;
use chrono::Utc;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub struct AuthService;

#[derive(Deserialize, Serialize)]
pub struct AuthTokensResponse {
    pub access_token: String,
    pub refresh_token: String,
}

impl AuthService {
    pub async fn login_with_password(
        ctx: &ServiceContext,
        email: String,
        password: String,
    ) -> Result<AuthTokensResponse, AppError> {
        let settings = &ctx.app_state.settings;
        let email = email.trim().to_lowercase();
        let (user_id, user_public_id) = match UserService::get_user_id_by_email(ctx, &email).await {
            Ok(user) => user,
            Err(AppError::User(UserServiceError::NotFound)) => {
                LoginLogsService::save_log(
                    ctx,
                    None,
                    email,
                    SignInLogEventType::LoginFailure,
                    None,
                )
                .await?;
                return Err(AppError::InvalidCredentials);
            }
            Err(error) => return Err(error),
        };
        let credential = UserCredentialService::get_credential(ctx, user_id).await?;

        if let Err(_) =
            UserCredentialService::verify_login_password(settings, &password, &credential).await
        {
            UserCredentialService::inc_failed_attempts(
                &ctx.app_state.primary_write_replica,
                user_id,
            )
            .await?;
            LoginLogsService::save_log(
                ctx,
                Some(user_id),
                email,
                SignInLogEventType::LoginFailure,
                None,
            )
            .await?;
            return Err(AppError::InvalidCredentials);
        }

        UserCredentialService::save_last_login_at(&ctx.app_state.primary_write_replica, user_id)
            .await?;
        UserCredentialService::reset_failed_attempts(&ctx.app_state.primary_write_replica, user_id)
            .await?;
        LoginLogsService::save_log(
            ctx,
            Some(user_id),
            email,
            SignInLogEventType::LoginSuccess,
            None,
        )
        .await?;
        Self::issue_auth_tokens(ctx, user_id, &user_public_id).await
    }

    pub async fn refresh_tokens(
        ctx: &ServiceContext,
        refresh_token: String,
    ) -> Result<AuthTokensResponse, AppError> {
        let settings = &ctx.app_state.settings;
        let jwt = JwtHelpers::new(settings);
        let claims = jwt.verify_refresh_token(&refresh_token)?;
        let user = UserService::get_user_by_public_id(ctx, &claims.public_id).await?;
        let credential = UserCredentialService::get_credential(ctx, user.id).await?;

        let is_match =
            UserCredentialService::verify_refresh_token(settings, &refresh_token, &credential)?;
        if !is_match {
            return Err(JwtError::InvalidToken.into());
        }

        if let Some(refresh_expiry) = credential.refresh_token_expires_at {
            if refresh_expiry < Utc::now() {
                return Err(JwtError::InvalidToken.into());
            }
        }

        if let Some(password_changed_at) = credential.password_changed_at {
            if claims.iat < password_changed_at.timestamp() as usize {
                return Err(JwtError::InvalidToken.into());
            }
        }

        let tokens = Self::issue_auth_tokens(ctx, user.id, &user.public_id).await?;
        LoginLogsService::save_log(
            ctx,
            Some(user.id),
            user.email,
            SignInLogEventType::RefreshToken,
            None,
        )
        .await?;
        Ok(tokens)
    }

    pub async fn logout(ctx: &ServiceContext) -> Result<(), AppError> {
        let user_id = ctx.get_user_id()?;
        let user = UserService::get_user_by_id(ctx, user_id).await?;

        UserCredentialService::clear_refresh_token(&ctx.app_state.primary_write_replica, user_id)
            .await?;

        LoginLogsService::save_log(
            ctx,
            Some(user_id),
            user.email,
            SignInLogEventType::Logout,
            None,
        )
        .await?;
        Ok(())
    }

    async fn issue_auth_tokens(
        ctx: &ServiceContext,
        user_id: i32,
        user_public_id: &str,
    ) -> Result<AuthTokensResponse, AppError> {
        let settings = &ctx.app_state.settings;
        let jwt = JwtHelpers::new(settings);
        let access_token = jwt.generate_access_token(user_public_id)?;
        let refresh_token = jwt.generate_refresh_token(user_public_id)?;
        let refresh_claims = jwt.verify_refresh_token(&refresh_token)?;

        UserCredentialService::save_refresh_token(
            settings,
            &ctx.app_state.primary_write_replica,
            user_id,
            &refresh_token,
            chrono::DateTime::from_timestamp(refresh_claims.exp as i64, 0)
                .ok_or(JwtError::CannotGenerateToken)?,
        )
        .await?;

        Ok(AuthTokensResponse {
            access_token,
            refresh_token,
        })
    }
}
