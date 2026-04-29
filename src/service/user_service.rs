use crate::entity::user_entity;
use crate::entity::user_entity::UserStatus;
use crate::service_cotext::ServiceContext;
use crate::utils::date_helpers::DateHelper;
use sea_orm::{ActiveModelTrait, Set};

pub struct CreateUser {
    first_name: String,
    last_name: String,
    email: String,
    password: String,
}

pub async fn create_user(ctx: &ServiceContext, payload: CreateUser) {
    let user = user_entity::ActiveModel {
        public_id: Set("ABC".to_owned()),
        first_name: Set(payload.first_name),
        last_name: Set(payload.last_name),
        email: Set(payload.email),
        email_verified: Set(false),
        status: Set(UserStatus::Active),
        created_at: Set(DateHelper::now().value()),
        updated_at: Set(DateHelper::now().value()),
        ..Default::default()
    };

    let user: user_entity::Model = user
        .insert(&ctx.app_state.primary_write_replica)
        .await
        .unwrap();
}
