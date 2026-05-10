use crate::entity::login_log_entity::{self as entity, RequestContext, SignInLogEventType};
use crate::errors::app_error::AppError;
use crate::service::service_context::ServiceContext;
use crate::utils::date_helpers::DateHelper;
use sea_orm::{ActiveModelTrait, Set};

pub struct LoginLogsService {}

impl LoginLogsService {
    pub async fn save_log(
        ctx: &ServiceContext,
        user_id: Option<i32>,
        identifier: String,
        event_type: SignInLogEventType,
        request_context: Option<RequestContext>,
    ) -> Result<(), AppError> {
        let now = DateHelper::now().value();
        let login_log = entity::ActiveModel {
            user_id: Set(user_id),
            identifier: Set(identifier),
            created_at: Set(now),
            event_type: Set(event_type),
            request_context: Set(request_context.unwrap_or_default()),
            ..Default::default()
        };
        login_log
            .insert(&ctx.app_state.primary_write_replica)
            .await?;
        Ok(())
    }
}
