use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};

use crate::api::dto::coa_dto::CreateChartOfAccountResolutionInput;
use crate::entity::coa::coa_entity as CoaAccount;
use crate::entity::{GenericStatus, PrimaryId};
use crate::errors::app_error::AppError;
use crate::errors::coa_service_errors::CoaServiceError;

pub struct CoaPayloadResolver;

impl CoaPayloadResolver {
    pub async fn create_account(
        db_transaction: &impl ConnectionTrait,
        organization_id: PrimaryId,
        payload: CreateChartOfAccountResolutionInput,
    ) -> Result<crate::service::coa_service::CreateChartOfAccountInput, AppError> {
        let parent_public_id = payload.parent_account_public_id.trim();
        let parent = CoaAccount::Entity::find()
            .filter(CoaAccount::Column::OrganizationId.eq(organization_id))
            .filter(CoaAccount::Column::PublicId.eq(parent_public_id))
            .filter(CoaAccount::Column::Status.eq(GenericStatus::Active))
            .one(db_transaction)
            .await?
            .ok_or(CoaServiceError::ParentAccountNotFound)?;

        Ok(payload.into_service_input(parent.id))
    }
}
