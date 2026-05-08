use crate::entity::login_log_entity::SignInLogEventType;
use crate::errors::app_error::AppError;
use crate::service::login_log_service::LoginLogsService;
use crate::service::service_context::ServiceContext;

pub struct AuthService {}

impl AuthService {
    async fn login_with_password(
        ctx: &ServiceContext,
        email: String,
        password: String,
    ) -> Result<bool, AppError> {


        // save the login record.
        LoginLogsService::save_log(
            ctx,
            Some(12),
            String::from("surojit99923@gmai.com"),
            SignInLogEventType::LoginSuccess,
            None,
        )
        .await?;
        Ok(true)
    }

}
