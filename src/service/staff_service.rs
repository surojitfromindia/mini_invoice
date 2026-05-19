use sea_orm::{ActiveModelTrait, ConnectionTrait, Set};
use crate::entity::{OrganizationPrimaryId, UserPrimaryId};
use crate::errors::app_error::AppError;
use crate::service::service_context::ServiceContext;
use crate::service::user_service::UserService;


use crate::entity::organization::staff_entity::{self as Staff, StaffStatus};
use crate::utils::date_helpers::DateHelper;
use crate::utils::id_generator::IdGenerator;

pub struct StaffService;





impl StaffService {
    pub async fn  create_staff_from_user(
        db_transaction: &impl ConnectionTrait,
        ctx: &ServiceContext,
        organization_id: OrganizationPrimaryId,
    )-> Result<(), AppError> {
        let actor_id = ctx.get_actor_id()?;
        let user_id = ctx.get_user_id()?;
        let user = UserService::get_user_by_id(
            &ctx,
            user_id,
        ).await?;

        let now = DateHelper::now().value();
        let public_id = IdGenerator::generate_general_id();


        let staff_active_model = Staff::ActiveModel {
            user_id: Set(user_id),
            organization_id: Set(organization_id),
            public_id: Set(public_id),
            name_primary: Set(format!("{} {}", user.first_name, user.last_name)),
            name_secondary: Set(None),
            status: Set(StaffStatus::Active),
            created_by_actor_id: Set(actor_id),
            updated_by_actor_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        staff_active_model
            .insert(db_transaction)
            .await?;


        Ok(())
    }
}