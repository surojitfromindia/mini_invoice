use std::collections::{HashMap, HashSet};

use sea_orm::entity::prelude::{DateTimeUtc, Decimal};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set,
    TransactionTrait,
};

use crate::entity::item::item_entity::{self as Item, ItemType};
use crate::entity::item::item_unit_entity as ItemUnit;
use crate::entity::item::unit_entity as Unit;
use crate::entity::{ActorPrimaryId, OrganizationPrimaryId, PublicId};
use crate::errors::app_error::AppError;
use crate::errors::item_service_errors::ItemServiceError;
use crate::service::service_context::ServiceContext;
use crate::utils::date_helpers::DateHelper;
use crate::utils::id_generator::IdGenerator;

pub struct CreateItemInput {
    pub sku: String,
    pub barcode: Option<String>,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub description: Option<String>,
    pub item_type: ItemType,
    pub default_purchase_price: Decimal,
    pub default_sales_price: Decimal,
    pub track_inventory: bool,
    pub allow_negative_stock: bool,
    pub reorder_level: Option<Decimal>,
    pub is_active: Option<bool>,
    pub unit_configurations: Vec<CreateItemUnitInput>,
}

pub struct CreateItemUnitInput {
    pub unit_public_id: String,
    pub conversion_factor_to_base: Decimal,
    pub is_base_unit: bool,
    pub is_purchase_unit: bool,
    pub is_sales_unit: bool,
}

struct PredefinedUnitSeed {
    code: &'static str,
    name_primary: &'static str,
    symbol: &'static str,
    decimal_places: i16,
}

#[derive(Debug, Clone)]
struct NormalizedItemUnitInput {
    unit_public_id: String,
    conversion_factor_to_base: Decimal,
    is_base_unit: bool,
    is_purchase_unit: bool,
    is_sales_unit: bool,
}

pub struct ItemService;

impl ItemService {
    pub async fn create_item(
        ctx: &ServiceContext,
        payload: CreateItemInput,
    ) -> Result<PublicId, AppError> {
        let actor_id = ctx.get_actor_id()?;
        let organization_id = ctx.get_organization_id()?;
        let sku = payload.sku.trim().to_string();
        let name_primary = payload.name_primary.trim().to_string();

        if sku.is_empty() {
            return Err(AppError::BadRequest {
                code: crate::errors::error_codes::ITEM_INVALID_SKU,
                message: "Item sku is required".to_string(),
            });
        }

        if name_primary.is_empty() {
            return Err(AppError::BadRequest {
                code: crate::errors::error_codes::ITEM_INVALID_NAME,
                message: "Item primary name is required".to_string(),
            });
        }

        let normalized_unit_configurations =
            Self::normalize_unit_configurations(payload.unit_configurations)?;

        let txn = ctx.app_state.primary_write_replica.begin().await?;

        // Resolve organization-scoped units before item creation so the item and
        // its unit configuration are committed together.
        let units_by_public_id = Self::load_units_by_public_id(
            &txn,
            organization_id,
            &normalized_unit_configurations,
        )
        .await?;

        let base_unit = normalized_unit_configurations
            .iter()
            .find(|config| config.is_base_unit)
            .and_then(|config| units_by_public_id.get(&config.unit_public_id))
            .ok_or(ItemServiceError::InvalidBaseUnitConfiguration)?;

        let now = DateHelper::now().value();
        let item = Item::ActiveModel {
            organization_id: Set(organization_id),
            public_id: Set(IdGenerator::generate_general_id()),
            sku: Set(sku),
            barcode: Set(payload.barcode),
            name_primary: Set(name_primary),
            name_secondary: Set(payload.name_secondary),
            description: Set(payload.description),
            item_type: Set(payload.item_type),
            base_unit_id: Set(base_unit.id),
            default_purchase_price: Set(payload.default_purchase_price),
            default_sales_price: Set(payload.default_sales_price),
            track_inventory: Set(payload.track_inventory),
            allow_negative_stock: Set(payload.allow_negative_stock),
            reorder_level: Set(payload.reorder_level),
            is_active: Set(payload.is_active.unwrap_or(true)),
            created_by_actor_id: Set(actor_id),
            updated_by_actor_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        Self::create_item_units(
            &txn,
            actor_id,
            item.id,
            &units_by_public_id,
            &normalized_unit_configurations,
            now,
        )
        .await?;

        txn.commit().await?;

        Ok(item.public_id)
    }

    pub async fn seed_default_units_for_organization(
        db_transaction: &impl ConnectionTrait,
        actor_id: ActorPrimaryId,
        organization_id: OrganizationPrimaryId,
    ) -> Result<(), AppError> {
        let existing_codes: HashSet<String> = Unit::Entity::find()
            .filter(Unit::Column::OrganizationId.eq(organization_id))
            .all(db_transaction)
            .await?
            .into_iter()
            .map(|unit| unit.code)
            .collect();

        let now = DateHelper::now().value();

        // Bootstrap a practical default catalog so items can be created without
        // forcing each organization to redefine common units first.
        for seed in Self::predefined_units() {
            if existing_codes.contains(seed.code) {
                continue;
            }

            Unit::ActiveModel {
                organization_id: Set(organization_id),
                public_id: Set(IdGenerator::generate_general_id()),
                code: Set(seed.code.to_string()),
                name_primary: Set(seed.name_primary.to_string()),
                name_secondary: Set(None),
                symbol: Set(Some(seed.symbol.to_string())),
                decimal_places: Set(seed.decimal_places),
                is_system_unit: Set(true),
                is_active: Set(true),
                created_by_actor_id: Set(actor_id),
                updated_by_actor_id: Set(None),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            }
            .insert(db_transaction)
            .await?;
        }

        Ok(())
    }

    async fn load_units_by_public_id(
        db_transaction: &impl ConnectionTrait,
        organization_id: OrganizationPrimaryId,
        unit_configurations: &[NormalizedItemUnitInput],
    ) -> Result<HashMap<String, Unit::Model>, AppError> {
        let unit_public_ids: Vec<String> = unit_configurations
            .iter()
            .map(|config| config.unit_public_id.clone())
            .collect();

        let units = Unit::Entity::find()
            .filter(Unit::Column::OrganizationId.eq(organization_id))
            .filter(Unit::Column::PublicId.is_in(unit_public_ids.clone()))
            .all(db_transaction)
            .await?;

        if units.len() != unit_public_ids.len() {
            return Err(ItemServiceError::UnitNotFound.into());
        }

        Ok(units
            .into_iter()
            .map(|unit| (unit.public_id.clone(), unit))
            .collect())
    }

    async fn create_item_units(
        db_transaction: &impl ConnectionTrait,
        actor_id: ActorPrimaryId,
        item_id: i32,
        units_by_public_id: &HashMap<String, Unit::Model>,
        unit_configurations: &[NormalizedItemUnitInput],
        now: DateTimeUtc,
    ) -> Result<(), AppError> {
        for config in unit_configurations {
            let unit = units_by_public_id
                .get(&config.unit_public_id)
                .ok_or(ItemServiceError::UnitNotFound)?;

            ItemUnit::ActiveModel {
                item_id: Set(item_id),
                unit_id: Set(unit.id),
                conversion_factor_to_base: Set(config.conversion_factor_to_base),
                is_base_unit: Set(config.is_base_unit),
                is_purchase_unit: Set(config.is_purchase_unit),
                is_sales_unit: Set(config.is_sales_unit),
                created_by_actor_id: Set(actor_id),
                updated_by_actor_id: Set(None),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            }
            .insert(db_transaction)
            .await?;
        }

        Ok(())
    }

    fn normalize_unit_configurations(
        unit_configurations: Vec<CreateItemUnitInput>,
    ) -> Result<Vec<NormalizedItemUnitInput>, AppError> {
        if unit_configurations.is_empty() {
            return Err(ItemServiceError::UnitConfigurationRequired.into());
        }

        let mut seen_unit_ids = HashSet::new();
        let mut normalized = Vec::with_capacity(unit_configurations.len());
        let mut base_unit_count = 0;

        for config in unit_configurations {
            let unit_public_id = config.unit_public_id.trim().to_string();
            if unit_public_id.is_empty() {
                return Err(ItemServiceError::UnitNotFound.into());
            }

            if !seen_unit_ids.insert(unit_public_id.clone()) {
                return Err(ItemServiceError::DuplicateUnitConfiguration.into());
            }

            if config.conversion_factor_to_base <= Decimal::ZERO {
                return Err(ItemServiceError::InvalidUnitConversionFactor.into());
            }

            if config.is_base_unit {
                base_unit_count += 1;
                if config.conversion_factor_to_base != Decimal::ONE {
                    return Err(ItemServiceError::InvalidBaseUnitConfiguration.into());
                }
            }

            normalized.push(NormalizedItemUnitInput {
                unit_public_id,
                conversion_factor_to_base: config.conversion_factor_to_base,
                is_base_unit: config.is_base_unit,
                is_purchase_unit: config.is_purchase_unit,
                is_sales_unit: config.is_sales_unit,
            });
        }

        if base_unit_count != 1 {
            return Err(ItemServiceError::InvalidBaseUnitConfiguration.into());
        }

        Ok(normalized)
    }

    fn predefined_units() -> &'static [PredefinedUnitSeed] {
        &[
            PredefinedUnitSeed {
                code: "PCS",
                name_primary: "Piece",
                symbol: "pcs",
                decimal_places: 0,
            },
            PredefinedUnitSeed {
                code: "BOX",
                name_primary: "Box",
                symbol: "box",
                decimal_places: 0,
            },
            PredefinedUnitSeed {
                code: "CTN",
                name_primary: "Carton",
                symbol: "ctn",
                decimal_places: 0,
            },
            PredefinedUnitSeed {
                code: "DOZ",
                name_primary: "Dozen",
                symbol: "doz",
                decimal_places: 0,
            },
            PredefinedUnitSeed {
                code: "KG",
                name_primary: "Kilogram",
                symbol: "kg",
                decimal_places: 3,
            },
            PredefinedUnitSeed {
                code: "G",
                name_primary: "Gram",
                symbol: "g",
                decimal_places: 0,
            },
            PredefinedUnitSeed {
                code: "LTR",
                name_primary: "Litre",
                symbol: "ltr",
                decimal_places: 3,
            },
            PredefinedUnitSeed {
                code: "ML",
                name_primary: "Millilitre",
                symbol: "ml",
                decimal_places: 0,
            },
            PredefinedUnitSeed {
                code: "MTR",
                name_primary: "Meter",
                symbol: "m",
                decimal_places: 3,
            },
            PredefinedUnitSeed {
                code: "CM",
                name_primary: "Centimeter",
                symbol: "cm",
                decimal_places: 1,
            },
            PredefinedUnitSeed {
                code: "SQFT",
                name_primary: "Square Foot",
                symbol: "sqft",
                decimal_places: 2,
            },
            PredefinedUnitSeed {
                code: "HRS",
                name_primary: "Hour",
                symbol: "hr",
                decimal_places: 2,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_unit_configurations_rejects_missing_base_unit() {
        let error = ItemService::normalize_unit_configurations(vec![CreateItemUnitInput {
            unit_public_id: "unit_1".to_string(),
            conversion_factor_to_base: Decimal::ONE,
            is_base_unit: false,
            is_purchase_unit: true,
            is_sales_unit: true,
        }])
        .unwrap_err();

        assert_eq!(
            error.meta().code,
            crate::errors::error_codes::ITEM_INVALID_BASE_UNIT_CONFIGURATION
        );
    }

    #[test]
    fn normalize_unit_configurations_rejects_duplicate_unit_public_ids() {
        let error = ItemService::normalize_unit_configurations(vec![
            CreateItemUnitInput {
                unit_public_id: "unit_1".to_string(),
                conversion_factor_to_base: Decimal::ONE,
                is_base_unit: true,
                is_purchase_unit: true,
                is_sales_unit: true,
            },
            CreateItemUnitInput {
                unit_public_id: "unit_1".to_string(),
                conversion_factor_to_base: Decimal::from(12),
                is_base_unit: false,
                is_purchase_unit: true,
                is_sales_unit: false,
            },
        ])
        .unwrap_err();

        assert_eq!(
            error.meta().code,
            crate::errors::error_codes::ITEM_DUPLICATE_UNIT_CONFIGURATION
        );
    }
}
