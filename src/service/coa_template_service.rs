use std::collections::HashMap;

use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};

use crate::entity::coa::coa_entity as CoaAccount;
use crate::entity::coa::coa_template_entity as CoaTemplate;
use crate::entity::{GenericStatus, PrimaryId};
use crate::errors::app_error::AppError;
use crate::service::coa_seed_catalog;
use crate::utils::date_helpers::DateHelper;
use crate::utils::id_generator::IdGenerator;

pub struct CoaTemplateService;

impl CoaTemplateService {
    pub async fn seed_default_coa(
        db_transaction: &impl ConnectionTrait,
        actor_id: PrimaryId,
        organization_id: PrimaryId,
        country_iso_code: impl Into<String>,
    ) -> Result<(), AppError> {
        let country_iso_code = country_iso_code.into();
        let template_seed = coa_seed_catalog::template_for_country(&country_iso_code);

        let template = Self::ensure_default_template(
            db_transaction,
            actor_id,
            organization_id,
            &template_seed,
            &country_iso_code,
        )
        .await?;

        Self::seed_default_accounts(
            db_transaction,
            actor_id,
            organization_id,
            template.id,
            &template_seed.accounts,
        )
        .await?;

        Ok(())
    }

    async fn ensure_default_template(
        db_transaction: &impl ConnectionTrait,
        actor_id: PrimaryId,
        organization_id: PrimaryId,
        template_seed: &coa_seed_catalog::CoaTemplateSeed,
        country_iso_code: &str,
    ) -> Result<CoaTemplate::Model, AppError> {
        if let Some(existing_template) = CoaTemplate::Entity::find()
            .filter(CoaTemplate::Column::OrganizationId.eq(organization_id))
            .filter(CoaTemplate::Column::IsDefault.eq(true))
            .one(db_transaction)
            .await?
        {
            return Ok(existing_template);
        }

        let now = DateHelper::now().value();

        CoaTemplate::ActiveModel {
            public_id: Set(IdGenerator::generate_general_id()),
            name_primary: Set(template_seed.name_primary.to_string()),
            name_secondary: Set(template_seed.name_secondary.map(str::to_string)),
            description: Set(template_seed.description.map(str::to_string)),
            country_iso_code: Set(
                template_seed
                    .country_iso_code
                    .unwrap_or(country_iso_code)
                    .to_string(),
            ),
            accounting_standard: Set(template_seed.accounting_standard.map(str::to_string)),
            is_default: Set(template_seed.is_default),
            organization_id: Set(organization_id),
            status: Set(GenericStatus::Active),
            created_by_actor_id: Set(actor_id),
            updated_by_actor_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(db_transaction)
        .await
        .map_err(Into::into)
    }

    async fn seed_default_accounts(
        db_transaction: &impl ConnectionTrait,
        actor_id: PrimaryId,
        organization_id: PrimaryId,
        template_id: PrimaryId,
        account_seeds: &[coa_seed_catalog::CoaAccountSeed],
    ) -> Result<(), AppError> {
        let existing_accounts = CoaAccount::Entity::find()
            .filter(CoaAccount::Column::CoaTemplateId.eq(template_id))
            .all(db_transaction)
            .await?;

        let mut account_ids_by_key: HashMap<String, PrimaryId> = HashMap::new();
        let mut account_ids_by_code: HashMap<String, PrimaryId> = HashMap::new();
        let seed_by_code: HashMap<&str, &coa_seed_catalog::CoaAccountSeed> =
            account_seeds.iter().map(|seed| (seed.code.as_str(), seed)).collect();

        for account in existing_accounts {
            account_ids_by_code.insert(account.code.clone(), account.id);

            if let Some(seed) = seed_by_code.get(account.code.as_str()) {
                account_ids_by_key.insert(seed.key.to_string(), account.id);
            }
        }

        let now = DateHelper::now().value();

        for seed in account_seeds {
            if account_ids_by_code.contains_key(seed.code.as_str()) {
                continue;
            }

            let (parent_account_id, account_group_id, account_type_id) = match seed.kind {
                coa_seed_catalog::CoaNodeKind::Group => (None, None, None),
                coa_seed_catalog::CoaNodeKind::Type => {
                    let parent_account_id = resolve_required_key(
                        seed.parent_key,
                        &account_ids_by_key,
                        "parent account",
                        seed.key,
                    )?;
                    (Some(parent_account_id), Some(parent_account_id), None)
                }
                coa_seed_catalog::CoaNodeKind::Account => {
                    let parent_account_id = resolve_required_key(
                        seed.parent_key,
                        &account_ids_by_key,
                        "parent account",
                        seed.key,
                    )?;
                    let account_group_id = resolve_required_key(
                        seed.account_group_key,
                        &account_ids_by_key,
                        "account group",
                        seed.key,
                    )?;
                    let account_type_id = resolve_required_key(
                        seed.account_type_key,
                        &account_ids_by_key,
                        "account type",
                        seed.key,
                    )?;
                    (
                        Some(parent_account_id),
                        Some(account_group_id),
                        Some(account_type_id),
                    )
                }
            };

            let created_account = CoaAccount::ActiveModel {
                public_id: Set(IdGenerator::generate_general_id()),
                organization_id: Set(organization_id),
                coa_template_id: Set(template_id),
                parent_account_id: Set(parent_account_id),
                account_group_id: Set(account_group_id),
                account_type_id: Set(account_type_id),
                code: Set(seed.code.clone()),
                name_primary: Set(seed.name_primary.to_string()),
                name_secondary: Set(seed.name_secondary.map(str::to_string)),
                description: Set(seed.description.map(str::to_string)),
                level_no: Set(seed.level_no),
                is_posting: Set(seed.is_posting),
                is_system_account: Set(seed.is_system_account),
                status: Set(GenericStatus::Active),
                created_by_actor_id: Set(actor_id),
                updated_by_actor_id: Set(None),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            }
            .insert(db_transaction)
            .await?;

            account_ids_by_key.insert(seed.key.to_string(), created_account.id);
            account_ids_by_code.insert(seed.code.clone(), created_account.id);
        }

        Ok(())
    }
}

fn resolve_required_key(
    key: Option<&'static str>,
    ids_by_key: &HashMap<String, PrimaryId>,
    label: &str,
    seed_key: &str,
) -> Result<PrimaryId, AppError> {
    match key {
        Some(key) => ids_by_key
            .get(key)
            .copied()
            .ok_or_else(|| {
                AppError::InternalServer(format!(
                    "Missing {label} `{key}` while seeding COA row `{seed_key}`"
                ))
            }),
        None => Err(AppError::InternalServer(format!(
            "Missing required {label} while seeding COA row `{seed_key}`"
        ))),
    }
}
