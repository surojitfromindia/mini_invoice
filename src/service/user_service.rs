use crate::entity::user_entity;
use crate::entity::user_entity::UserStatus;
use crate::service_cotext::ServiceContext;
use crate::utils::date_helpers::DateHelper;
use crate::utils::id_generator::IdGenerator;
use sea_orm::{ActiveModelTrait, Set};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateUserAccount {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password: String,
}


pub async fn create_user_account(ctx: &ServiceContext, payload: CreateUserAccount) {
    let public_id = IdGenerator::get_user_id();
    let user = user_entity::ActiveModel {
        public_id: Set(public_id),
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
