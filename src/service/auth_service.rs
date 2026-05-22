use crate::entity::login_log_entity::SignInLogEventType;
use crate::errors::app_error::AppError;
use crate::errors::user_service_errors::UserServiceError;
use crate::service::login_log_service::LoginLogsService;
use crate::service::service_context::ServiceContext;
use crate::service::user_credential_service::UserCredentialService;
use crate::service::user_service::UserService;
use crate::utils::jwt_helpers::JwtHelpers;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub struct AuthService;

#[derive(Deserialize, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
}

impl AuthService {
    pub async fn login_with_password(
        ctx: &ServiceContext,
        email: String,
        password: String,
    ) -> Result<LoginResponse, AppError> {
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

        let jwt = JwtHelpers::new(settings);
        let access_token = jwt.generate_access_token(&user_public_id)?;
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
        Ok(LoginResponse { access_token })
    }
}
