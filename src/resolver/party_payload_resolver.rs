use std::collections::HashMap;

use sea_orm::ConnectionTrait;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::api::dto::common_dto::IntoServiceInput;
use crate::api::dto::party_dto::{
    CreatePartyAccountingProfileRequestDto, CreatePartyAccountingProfileResolutionOutput,
    CreatePartyResolutionInput,
};
use crate::entity::coa::coa_entity as CoaAccount;
use crate::entity::{GenericStatus, PrimaryId};
use crate::errors::app_error::AppError;
use crate::errors::error_codes;
use crate::resolver::public_id_resolver::PublicIdResolver;
use crate::service::party_service::{CreatePartyAccountingProfileInput, CreatePartyInput};

pub struct PartyPayloadResolver;

impl PartyPayloadResolver {
    pub async fn create_party(
        db_transaction: &impl ConnectionTrait,
        organization_id: PrimaryId,
        payload: CreatePartyResolutionInput,
    ) -> Result<CreatePartyInput, AppError> {
        let branch_public_ids = payload
            .branch_public_id
            .map(|public_id| vec![public_id])
            .filter(|public_ids| !public_ids.is_empty());
        let branch_ids = PublicIdResolver::branch_ids(
            db_transaction,
            organization_id,
            branch_public_ids.as_deref(),
        )
        .await?;
        let branch_id = branch_ids
            .into_iter()
            .next()
            .ok_or_else(|| AppError::InternalServer("Failed to resolve party branch".into()))?;
        let accounting_profile = match payload.accounting_profile {
            Some(accounting_profile) => Some(
                Self::resolve_accounting_profile(
                    db_transaction,
                    organization_id,
                    accounting_profile,
                )
                .await?
                .into(),
            ),
            None => None,
        };

        Ok(CreatePartyInput {
            branch_id,
            party_type: payload.party_type.into_service_input(),
            party_kind: payload.party_kind.into_service_input(),
            status: payload.status.map(|status| status.into_service_input()),
            source: payload.source.map(|source| source.into_service_input()),
            display_name: payload.display_name,
            name_primary: payload.name_primary,
            name_secondary: payload.name_secondary,
            legal_name: payload.legal_name,
            phone: payload.phone,
            email: payload.email,
            tax_no: payload.tax_no,
            tax_treatment: payload.tax_treatment,
            country_iso_code: payload.country_iso_code,
            currency_iso_code: payload.currency_iso_code,
            payment_terms_days: payload.payment_terms_days,
            credit_limit: payload.credit_limit,
            allow_credit: payload.allow_credit,
            notes: payload.notes,
            addresses: payload
                .addresses
                .into_iter()
                .map(|address| address.into_service_input())
                .collect(),
            contacts: payload
                .contacts
                .into_iter()
                .map(|contact| contact.into_service_input())
                .collect(),
            accounting_profile,
        })
    }

    async fn resolve_accounting_profile(
        db_transaction: &impl ConnectionTrait,
        organization_id: PrimaryId,
        payload: CreatePartyAccountingProfileRequestDto,
    ) -> Result<CreatePartyAccountingProfileInput, AppError> {
        let public_ids = collect_account_public_ids(&payload);
        let accounts_by_public_id =
            Self::fetch_posting_accounts_by_public_id(db_transaction, organization_id, &public_ids)
                .await?;
        let resolved = CreatePartyAccountingProfileResolutionOutput {
            default_sales_account_id: resolve_optional_account_id(
                payload.default_sales_account_public_id.as_deref(),
                &accounts_by_public_id,
            )?,
            default_purchase_account_id: resolve_optional_account_id(
                payload.default_purchase_account_public_id.as_deref(),
                &accounts_by_public_id,
            )?,
            default_receivable_account_id: resolve_optional_account_id(
                payload.default_receivable_account_public_id.as_deref(),
                &accounts_by_public_id,
            )?,
            default_payable_account_id: resolve_optional_account_id(
                payload.default_payable_account_public_id.as_deref(),
                &accounts_by_public_id,
            )?,
            default_output_tax_account_id: resolve_optional_account_id(
                payload.default_output_tax_account_public_id.as_deref(),
                &accounts_by_public_id,
            )?,
            default_input_tax_account_id: resolve_optional_account_id(
                payload.default_input_tax_account_public_id.as_deref(),
                &accounts_by_public_id,
            )?,
        };

        Ok(resolved.into())
    }

    async fn fetch_posting_accounts_by_public_id(
        db_transaction: &impl ConnectionTrait,
        organization_id: PrimaryId,
        public_ids: &[String],
    ) -> Result<HashMap<String, PrimaryId>, AppError> {
        if public_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let accounts = CoaAccount::Entity::find()
            .filter(CoaAccount::Column::OrganizationId.eq(organization_id))
            .filter(CoaAccount::Column::PublicId.is_in(public_ids.iter().cloned()))
            .filter(CoaAccount::Column::Status.eq(GenericStatus::Active))
            .all(db_transaction)
            .await?;

        if accounts.len() != public_ids.len() {
            return Err(AppError::BadRequest {
                code: error_codes::PARTY_ACCOUNT_NOT_FOUND,
                message: "One or more party accounting profile accounts were not found".into(),
            });
        }

        if accounts.iter().any(|account| !account.is_posting) {
            return Err(AppError::BadRequest {
                code: error_codes::PARTY_ACCOUNT_NOT_POSTING,
                message: "Party accounting profile accounts must be posting accounts".into(),
            });
        }

        Ok(accounts
            .into_iter()
            .map(|account| (account.public_id, account.id))
            .collect())
    }
}

fn collect_account_public_ids(payload: &CreatePartyAccountingProfileRequestDto) -> Vec<String> {
    let mut public_ids = Vec::new();

    push_optional_public_id(
        &mut public_ids,
        payload.default_sales_account_public_id.as_deref(),
    );
    push_optional_public_id(
        &mut public_ids,
        payload.default_purchase_account_public_id.as_deref(),
    );
    push_optional_public_id(
        &mut public_ids,
        payload.default_receivable_account_public_id.as_deref(),
    );
    push_optional_public_id(
        &mut public_ids,
        payload.default_payable_account_public_id.as_deref(),
    );
    push_optional_public_id(
        &mut public_ids,
        payload.default_output_tax_account_public_id.as_deref(),
    );
    push_optional_public_id(
        &mut public_ids,
        payload.default_input_tax_account_public_id.as_deref(),
    );

    public_ids.sort();
    public_ids.dedup();
    public_ids
}

fn push_optional_public_id(public_ids: &mut Vec<String>, public_id: Option<&str>) {
    if let Some(public_id) = public_id
        .map(str::trim)
        .filter(|public_id| !public_id.is_empty())
    {
        public_ids.push(public_id.to_string());
    }
}

fn resolve_optional_account_id(
    public_id: Option<&str>,
    accounts_by_public_id: &HashMap<String, PrimaryId>,
) -> Result<Option<PrimaryId>, AppError> {
    let Some(public_id) = public_id
        .map(str::trim)
        .filter(|public_id| !public_id.is_empty())
    else {
        return Ok(None);
    };

    accounts_by_public_id
        .get(public_id)
        .copied()
        .map(Some)
        .ok_or_else(|| AppError::BadRequest {
            code: error_codes::PARTY_ACCOUNT_NOT_FOUND,
            message: "One or more party accounting profile accounts were not found".into(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_account_public_ids_trims_filters_and_deduplicates_values() {
        let payload = CreatePartyAccountingProfileRequestDto {
            default_sales_account_public_id: Some(" sales ".to_string()),
            default_purchase_account_public_id: None,
            default_receivable_account_public_id: Some("".to_string()),
            default_payable_account_public_id: Some("payable".to_string()),
            default_output_tax_account_public_id: Some("sales".to_string()),
            default_input_tax_account_public_id: Some("   ".to_string()),
        };

        let public_ids = collect_account_public_ids(&payload);

        assert_eq!(public_ids, vec!["payable".to_string(), "sales".to_string()]);
    }

    #[test]
    fn resolve_optional_account_id_accepts_empty_optional_values() {
        let accounts_by_public_id = HashMap::new();

        let resolved = resolve_optional_account_id(Some("   "), &accounts_by_public_id)
            .expect("empty optional account public id should resolve to none");

        assert_eq!(resolved, None);
    }
}
