use std::collections::HashSet;

use sea_orm::FromQueryResult;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};

use crate::db::listing::PageListResult;
use crate::db::listing::{execute_page_query, validate_page_pagination};
use crate::entity::item::unit_entity::{self as Unit};
use crate::entity::{GenericStatus, PrimaryId, PublicId};
use crate::errors::app_error::AppError;
use crate::service::service_context::ServiceContext;
use crate::utils::date_helpers::DateHelper;
use crate::utils::id_generator::IdGenerator;
use crate::utils::misc_helpers::trim_and_filter_empty;

pub struct CreateUnitInput {
    pub code: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub symbol: Option<String>,
    pub decimal_places: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitSortField {
    CreatedAt,
    Code,
    NamePrimary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

pub struct UnitListPageInput {
    pub page: u64,
    pub per_page: u64,
    pub code: Option<String>,
    pub name: Option<String>,
    pub status: Option<GenericStatus>,
    pub is_system_unit: Option<bool>,
    pub sort: Option<UnitSortField>,
    pub direction: Option<SortDirection>,
}

#[derive(Debug, Clone, PartialEq, FromQueryResult)]
pub struct UnitListItem {
    pub public_id: String,
    pub code: String,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub symbol: Option<String>,
    pub decimal_places: i16,
    pub is_system_unit: bool,
    pub status: GenericStatus,
}

struct PredefinedUnitSeed {
    code: &'static str,
    name_primary: &'static str,
    symbol: &'static str,
    decimal_places: i16,
}

pub struct UnitService;

impl UnitService {
    pub async fn create_unit(
        ctx: &ServiceContext,
        payload: CreateUnitInput,
    ) -> Result<PublicId, AppError> {
        let actor_id = ctx.get_actor_id()?;
        let organization_id = ctx.get_organization_id()?;
        let now = DateHelper::now().value();

        let unit = Unit::ActiveModel {
            organization_id: Set(organization_id),
            public_id: Set(IdGenerator::generate_general_id()),
            code: Set(payload.code.trim().to_uppercase()),
            name_primary: Set(payload.name_primary),
            name_secondary: Set(payload.name_secondary),
            symbol: Set(payload.symbol),
            decimal_places: Set(payload.decimal_places),
            is_system_unit: Set(false),
            status: Set(GenericStatus::Active),
            created_by_actor_id: Set(actor_id),
            updated_by_actor_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&ctx.app_state.primary_write_replica)
        .await?;

        Ok(unit.public_id)
    }

    pub async fn list_units_page(
        ctx: &ServiceContext,
        input: UnitListPageInput,
    ) -> Result<PageListResult<UnitListItem>, AppError> {
        let pagination = validate_page_pagination(input.page, input.per_page)?;
        let organization_id = ctx.get_organization_id()?;
        let sort_field = input.sort.unwrap_or(UnitSortField::CreatedAt);
        let sort_direction = input.direction.unwrap_or(SortDirection::Desc);

        let query = Self::build_unit_list_query(
            organization_id,
            input.code.as_deref(),
            input.name.as_deref(),
            input.status,
            input.is_system_unit,
        );
        let query = Self::apply_page_sort(query, sort_field, sort_direction);
        let query = Self::select_unit_list_columns(query).into_model::<UnitListItem>();

        let result =
            execute_page_query(&ctx.app_state.primary_read_replica, query, pagination).await?;

        Ok(result)
    }

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

    fn build_unit_list_query(
        organization_id: PrimaryId,
        code: Option<&str>,
        name: Option<&str>,
        status: Option<GenericStatus>,
        is_system_unit: Option<bool>,
    ) -> sea_orm::Select<Unit::Entity> {
        let mut query =
            Unit::Entity::find().filter(Unit::Column::OrganizationId.eq(organization_id));

        if let Some(code) = trim_and_filter_empty(code) {
            query = query.filter(Unit::Column::Code.contains(code.to_uppercase()));
        }

        if let Some(name) = trim_and_filter_empty(name) {
            query = query.filter(Unit::Column::NamePrimary.contains(name));
        }

        if let Some(status) = status {
            query = query.filter(Unit::Column::Status.eq(status));
        } else {
            query = query.filter(Unit::Column::Status.ne(GenericStatus::Deleted));
        }

        if let Some(is_system_unit) = is_system_unit {
            query = query.filter(Unit::Column::IsSystemUnit.eq(is_system_unit));
        }

        query
    }

    fn apply_page_sort(
        query: sea_orm::Select<Unit::Entity>,
        sort_field: UnitSortField,
        sort_direction: SortDirection,
    ) -> sea_orm::Select<Unit::Entity> {
        match (sort_field, sort_direction) {
            (UnitSortField::CreatedAt, SortDirection::Asc) => query
                .order_by_asc(Unit::Column::CreatedAt)
                .order_by_asc(Unit::Column::Id),
            (UnitSortField::CreatedAt, SortDirection::Desc) => query
                .order_by_desc(Unit::Column::CreatedAt)
                .order_by_desc(Unit::Column::Id),
            (UnitSortField::Code, SortDirection::Asc) => query
                .order_by_asc(Unit::Column::Code)
                .order_by_asc(Unit::Column::Id),
            (UnitSortField::Code, SortDirection::Desc) => query
                .order_by_desc(Unit::Column::Code)
                .order_by_desc(Unit::Column::Id),
            (UnitSortField::NamePrimary, SortDirection::Asc) => query
                .order_by_asc(Unit::Column::NamePrimary)
                .order_by_asc(Unit::Column::Id),
            (UnitSortField::NamePrimary, SortDirection::Desc) => query
                .order_by_desc(Unit::Column::NamePrimary)
                .order_by_desc(Unit::Column::Id),
        }
    }

    fn select_unit_list_columns<Q>(query: Q) -> Q
    where
        Q: QuerySelect<QueryStatement = sea_orm::sea_query::SelectStatement>,
    {
        query
            .select_only()
            .column(Unit::Column::PublicId)
            .column(Unit::Column::Code)
            .column(Unit::Column::NamePrimary)
            .column(Unit::Column::NameSecondary)
            .column(Unit::Column::Symbol)
            .column(Unit::Column::DecimalPlaces)
            .column(Unit::Column::IsSystemUnit)
            .column(Unit::Column::Status)
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
