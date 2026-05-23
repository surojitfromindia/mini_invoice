use crate::entity::actor_entity::{self as Actor, ActorEntity, ActorModel, ActorType};
use crate::entity::{ClientAppPrimaryId, PublicId, UserPrimaryId};
use crate::errors::app_error::AppError;
use crate::utils::date_helpers::DateHelper;
use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};

pub struct ActorService {}

impl ActorService {
    pub async fn create_from_user(
        db_transaction: &impl ConnectionTrait,
        user_id: UserPrimaryId,
        public_id: PublicId,
    ) -> Result<(), AppError> {
        let now = DateHelper::now().value();
        let actor_active_model = Actor::ActiveModel {
            user_id: Set(Some(user_id)),
            public_user_id: Set(Some(public_id)),
            client_app_id: Set(None),
            public_client_app_id: Set(None),
            actor_type: Set(ActorType::User),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        actor_active_model.insert(db_transaction).await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn create_from_client_app(
        db_transaction: &impl ConnectionTrait,
        client_app_id: ClientAppPrimaryId,
        public_id: PublicId,
    ) -> Result<(), AppError> {
        let now = DateHelper::now().value();
        let actor_active_model = Actor::ActiveModel {
            user_id: Set(None),
            public_user_id: Set(None),
            client_app_id: Set(Some(client_app_id)),
            public_client_app_id: Set(Some(public_id)),
            actor_type: Set(ActorType::ClientApp),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };
        actor_active_model.insert(db_transaction).await?;
        Ok(())
    }
}
