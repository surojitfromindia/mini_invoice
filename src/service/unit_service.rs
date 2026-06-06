use std::collections::HashSet;

use sea_orm::{ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, Set};

use crate::entity::item::unit_entity::{self as Unit};
use crate::entity::{GenericStatus, PrimaryId};
use crate::errors::app_error::AppError;
use crate::utils::date_helpers::DateHelper;
use crate::utils::id_generator::IdGenerator;

struct PredefinedUnitSeed {
    code: &'static str,
    name_primary: &'static str,
    symbol: &'static str,
    decimal_places: i16,
}

pub struct UnitService;

impl UnitService {
    // Bootstrap a practical default catalog so organizations can create items
    // immediately without manually defining the most common units first.
    pub async fn seed_default_units_for_organization(
        db_transaction: &impl ConnectionTrait,
        actor_id: PrimaryId,
        organization_id: PrimaryId,
    ) -> Result<(), AppError> {
        let existing_codes: HashSet<String> = Unit::Entity::find()
            .filter(Unit::Column::OrganizationId.eq(organization_id))
            .all(db_transaction)
            .await?
            .into_iter()
            .map(|unit| unit.code)
            .collect();

        let now = DateHelper::now().value();

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
                status: Set(GenericStatus::Active),
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
