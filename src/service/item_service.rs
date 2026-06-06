use sea_orm::entity::prelude::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, FromQueryResult, QueryFilter, QueryOrder,
    QuerySelect, Set, TransactionTrait,
};

use crate::db::listing::PageListResult;
use crate::db::listing::{execute_page_query, validate_page_pagination};
use crate::entity::item::item_entity::{self as Item, ItemStatus, ItemType, ItemUsage};
use crate::entity::{PublicId, UnitPrimaryId};
use crate::errors::app_error::AppError;
use crate::service::service_context::ServiceContext;
use crate::utils::date_helpers::DateHelper;
use crate::utils::id_generator::IdGenerator;
use crate::utils::misc_helpers::trim_and_filter_empty;

pub struct CreateItemInput {
    pub sku: String,
    pub barcode: Option<String>,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub description: Option<String>,
    pub item_type: ItemType,
    pub item_usage: ItemUsage,
    pub base_unit_id: UnitPrimaryId,
    pub purchase_unit_id: Option<UnitPrimaryId>,
    pub sales_unit_id: Option<UnitPrimaryId>,
    pub default_purchase_price: Option<Decimal>,
    pub default_sales_price: Option<Decimal>,
    pub track_inventory: bool,
    pub allow_negative_stock: bool,
    pub reorder_level: Option<Decimal>,
    pub status: Option<ItemStatus>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemSortField {
    CreatedAt,
    NamePrimary,
    Sku,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

pub struct ItemListPageInput {
    pub page: u64,
    pub per_page: u64,
    pub name: Option<String>,
    pub sku: Option<String>,
    pub status: Option<ItemStatus>,
    pub item_type: Option<ItemType>,
    pub sort: Option<ItemSortField>,
    pub direction: Option<SortDirection>,
}

#[derive(Debug, Clone, PartialEq, FromQueryResult)]
pub struct ItemListItem {
    pub public_id: String,
    pub sku: String,
    pub barcode: Option<String>,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub item_type: ItemType,
    pub item_usage: ItemUsage,
    pub default_purchase_price: Option<Decimal>,
    pub default_sales_price: Option<Decimal>,
    pub track_inventory: bool,
    pub status: ItemStatus,
}

pub struct ItemService;

impl ItemService {
    pub async fn create_item(
        ctx: &ServiceContext,
        payload: CreateItemInput,
    ) -> Result<PublicId, AppError> {
        let actor_id = ctx.get_actor_id()?;
        let organization_id = ctx.get_organization_id()?;

        let txn = ctx.app_state.primary_write_replica.begin().await?;

        let now = DateHelper::now().value();
        let item = Item::ActiveModel {
            organization_id: Set(organization_id),
            public_id: Set(IdGenerator::generate_general_id()),
            sku: Set(payload.sku),
            barcode: Set(payload.barcode),
            name_primary: Set(payload.name_primary),
            name_secondary: Set(payload.name_secondary),
            description: Set(payload.description),
            item_type: Set(payload.item_type),
            item_usage: Set(payload.item_usage),
            base_unit_id: Set(payload.base_unit_id),
            purchase_unit_id: Set(payload.purchase_unit_id),
            sales_unit_id: Set(payload.sales_unit_id),
            default_purchase_price: Set(payload.default_purchase_price),
            default_sales_price: Set(payload.default_sales_price),
            track_inventory: Set(payload.track_inventory),
            allow_negative_stock: Set(payload.allow_negative_stock),
            reorder_level: Set(payload.reorder_level),
            status: Set(ItemStatus::Active),
            created_by_actor_id: Set(actor_id),
            updated_by_actor_id: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        }
        .insert(&txn)
        .await?;

        txn.commit().await?;

        Ok(item.public_id)
    }

    pub async fn list_items_page(
        ctx: &ServiceContext,
        input: ItemListPageInput,
    ) -> Result<PageListResult<ItemListItem>, AppError> {
        let pagination = validate_page_pagination(input.page, input.per_page)?;
        let organization_id = ctx.get_organization_id()?;
        let sort_field = input.sort.unwrap_or(ItemSortField::CreatedAt);
        let sort_direction = input.direction.unwrap_or(SortDirection::Desc);

        let query = Self::build_item_list_query(
            organization_id,
            input.name.as_deref(),
            input.sku.as_deref(),
            input.status,
            input.item_type,
        );
        let query = Self::apply_page_sort(query, sort_field, sort_direction);
        let query = Self::select_item_list_columns(query).into_model::<ItemListItem>();

        execute_page_query(&ctx.app_state.primary_read_replica, query, pagination).await
    }

    fn build_item_list_query(
        organization_id: i32,
        name: Option<&str>,
        sku: Option<&str>,
        status: Option<ItemStatus>,
        item_type: Option<ItemType>,
    ) -> sea_orm::Select<Item::Entity> {
        let mut query =
            Item::Entity::find().filter(Item::Column::OrganizationId.eq(organization_id));

        if let Some(name) = trim_and_filter_empty(name) {
            query = query.filter(Item::Column::NamePrimary.contains(name));
        }

        if let Some(sku) = trim_and_filter_empty(sku) {
            query = query.filter(Item::Column::Sku.contains(sku));
        }

        if let Some(status) = status {
            query = query.filter(Item::Column::Status.eq(status));
        } else {
            query = query.filter(Item::Column::Status.ne(ItemStatus::Deleted));
        }

        if let Some(item_type) = item_type {
            query = query.filter(Item::Column::ItemType.eq(item_type));
        }

        query
    }

    fn apply_page_sort(
        query: sea_orm::Select<Item::Entity>,
        sort_field: ItemSortField,
        sort_direction: SortDirection,
    ) -> sea_orm::Select<Item::Entity> {
        match (sort_field, sort_direction) {
            (ItemSortField::CreatedAt, SortDirection::Asc) => query
                .order_by_asc(Item::Column::CreatedAt)
                .order_by_asc(Item::Column::Id),
            (ItemSortField::CreatedAt, SortDirection::Desc) => query
                .order_by_desc(Item::Column::CreatedAt)
                .order_by_desc(Item::Column::Id),
            (ItemSortField::NamePrimary, SortDirection::Asc) => query
                .order_by_asc(Item::Column::NamePrimary)
                .order_by_asc(Item::Column::Id),
            (ItemSortField::NamePrimary, SortDirection::Desc) => query
                .order_by_desc(Item::Column::NamePrimary)
                .order_by_desc(Item::Column::Id),
            (ItemSortField::Sku, SortDirection::Asc) => query
                .order_by_asc(Item::Column::Sku)
                .order_by_asc(Item::Column::Id),
            (ItemSortField::Sku, SortDirection::Desc) => query
                .order_by_desc(Item::Column::Sku)
                .order_by_desc(Item::Column::Id),
        }
    }

    fn select_item_list_columns<Q>(query: Q) -> Q
    where
        Q: QuerySelect<QueryStatement = sea_orm::sea_query::SelectStatement>,
    {
        query
            .select_only()
            .column(Item::Column::PublicId)
            .column(Item::Column::Sku)
            .column(Item::Column::Barcode)
            .column(Item::Column::NamePrimary)
            .column(Item::Column::NameSecondary)
            .column(Item::Column::ItemType)
            .column(Item::Column::ItemUsage)
            .column(Item::Column::DefaultPurchasePrice)
            .column(Item::Column::DefaultSalesPrice)
            .column(Item::Column::TrackInventory)
            .column(Item::Column::Status)
    }
}
