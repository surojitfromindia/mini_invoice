use crate::db::listing::PageListResult;
use crate::entity::item::item_entity::ItemType;
use sea_orm::entity::prelude::Decimal;
use serde::{Deserialize, Serialize};

use super::common_dto::{IntoServiceInput, PagePaginationQuery};
use crate::service::item_service::{ItemListItem, ItemListPageInput, ItemSortField, SortDirection};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemTypeDto {
    Inventory,
    Service,
    NonInventory,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemSortFieldDto {
    CreatedAt,
    NamePrimary,
    Sku,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortDirectionDto {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateItemRequestDto {
    pub sku: String,
    pub barcode: Option<String>,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub description: Option<String>,
    pub item_type: ItemTypeDto,
    pub base_unit_public_id: String,
    pub purchase_unit_public_id: String,
    pub sales_unit_public_id: String,
    pub default_purchase_price: Decimal,
    pub default_sales_price: Decimal,
    pub track_inventory: bool,
    pub allow_negative_stock: bool,
    pub reorder_level: Option<Decimal>,
    pub is_active: Option<bool>,
}

pub struct CreateItemResolutionInput {
    pub sku: String,
    pub barcode: Option<String>,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub description: Option<String>,
    pub item_type: ItemTypeDto,
    pub base_unit_public_id: String,
    pub purchase_unit_public_id: String,
    pub sales_unit_public_id: String,
    pub default_purchase_price: Decimal,
    pub default_sales_price: Decimal,
    pub track_inventory: bool,
    pub allow_negative_stock: bool,
    pub reorder_level: Option<Decimal>,
    pub is_active: Option<bool>,
}

impl CreateItemRequestDto {
    pub fn into_resolution_input(self) -> CreateItemResolutionInput {
        CreateItemResolutionInput {
            sku: self.sku,
            barcode: self.barcode,
            name_primary: self.name_primary,
            name_secondary: self.name_secondary,
            description: self.description,
            item_type: self.item_type,
            base_unit_public_id: self.base_unit_public_id,
            purchase_unit_public_id: self.purchase_unit_public_id,
            sales_unit_public_id: self.sales_unit_public_id,
            default_purchase_price: self.default_purchase_price,
            default_sales_price: self.default_sales_price,
            track_inventory: self.track_inventory,
            allow_negative_stock: self.allow_negative_stock,
            reorder_level: self.reorder_level,
            is_active: self.is_active,
        }
    }
}

impl ItemTypeDto {
    pub fn into_service_input(self) -> ItemType {
        match self {
            Self::Inventory => ItemType::Inventory,
            Self::Service => ItemType::Service,
            Self::NonInventory => ItemType::NonInventory,
        }
    }

    pub fn from_service_output(item_type: ItemType) -> Self {
        match item_type {
            ItemType::Inventory => Self::Inventory,
            ItemType::Service => Self::Service,
            ItemType::NonInventory => Self::NonInventory,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemListPageQueryDto {
    #[serde(flatten)]
    pub pagination: PagePaginationQuery,
    pub name: Option<String>,
    pub sku: Option<String>,
    pub is_active: Option<bool>,
    pub item_type: Option<ItemTypeDto>,
    pub sort: Option<ItemSortFieldDto>,
    pub direction: Option<SortDirectionDto>,
}

impl IntoServiceInput<ItemListPageInput> for ItemListPageQueryDto {
    fn into_service_input(self) -> ItemListPageInput {
        ItemListPageInput {
            page: self.pagination.page,
            per_page: self.pagination.per_page,
            name: self.name,
            sku: self.sku,
            is_active: self.is_active,
            item_type: self.item_type.map(ItemTypeDto::into_service_input),
            sort: self.sort.map(|sort| match sort {
                ItemSortFieldDto::CreatedAt => ItemSortField::CreatedAt,
                ItemSortFieldDto::NamePrimary => ItemSortField::NamePrimary,
                ItemSortFieldDto::Sku => ItemSortField::Sku,
            }),
            direction: self.direction.map(|direction| match direction {
                SortDirectionDto::Asc => SortDirection::Asc,
                SortDirectionDto::Desc => SortDirection::Desc,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemListItemResponseDto {
    pub public_id: String,
    pub sku: String,
    pub barcode: Option<String>,
    pub name_primary: String,
    pub name_secondary: Option<String>,
    pub item_type: ItemTypeDto,
    pub default_purchase_price: Decimal,
    pub default_sales_price: Decimal,
    pub track_inventory: bool,
    pub is_active: bool,
}

impl ItemListItemResponseDto {
    pub fn from_service_output(item: ItemListItem) -> Self {
        Self {
            public_id: item.public_id,
            sku: item.sku,
            barcode: item.barcode,
            name_primary: item.name_primary,
            name_secondary: item.name_secondary,
            item_type: ItemTypeDto::from_service_output(item.item_type),
            default_purchase_price: item.default_purchase_price,
            default_sales_price: item.default_sales_price,
            track_inventory: item.track_inventory,
            is_active: item.is_active,
        }
    }

    pub fn page_from_service_output(result: PageListResult<ItemListItem>) -> PageListResult<Self> {
        result.map_rows(Self::from_service_output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_item_request_deserializes_item_type_values() {
        let request: CreateItemRequestDto = serde_json::from_value(serde_json::json!({
            "sku": "ITEM-001",
            "barcode": "12345",
            "namePrimary": "Milk",
            "nameSecondary": "Full Cream",
            "description": "Shelf item",
            "itemType": "nonInventory",
            "baseUnitPublicId": "u_base",
            "purchaseUnitPublicId": "u_purchase",
            "salesUnitPublicId": "u_sales",
            "defaultPurchasePrice": "10.50",
            "defaultSalesPrice": "12.75",
            "trackInventory": true,
            "allowNegativeStock": false,
            "reorderLevel": "4.00",
            "isActive": true
        }))
        .unwrap();

        assert_eq!(request.item_type, ItemTypeDto::NonInventory);
        assert_eq!(request.base_unit_public_id, "u_base");
    }

    #[test]
    fn item_list_page_query_deserializes_filters_and_sort() {
        let query: ItemListPageQueryDto = serde_json::from_value(serde_json::json!({
            "page": 1,
            "perPage": 20,
            "name": "Milk",
            "sku": "ITEM",
            "isActive": true,
            "itemType": "inventory",
            "sort": "sku",
            "direction": "asc"
        }))
        .unwrap();

        assert_eq!(query.pagination.page, 1);
        assert_eq!(query.pagination.per_page, 20);
        assert_eq!(query.name.as_deref(), Some("Milk"));
        assert_eq!(query.sku.as_deref(), Some("ITEM"));
        assert_eq!(query.is_active, Some(true));
        assert_eq!(query.item_type, Some(ItemTypeDto::Inventory));
        assert_eq!(query.sort, Some(ItemSortFieldDto::Sku));
        assert_eq!(query.direction, Some(SortDirectionDto::Asc));
    }

    #[test]
    fn item_list_item_response_serializes_camel_case_keys() {
        let response = ItemListItemResponseDto {
            public_id: "item_123".to_string(),
            sku: "ITEM-001".to_string(),
            barcode: Some("12345".to_string()),
            name_primary: "Milk".to_string(),
            name_secondary: Some("Full Cream".to_string()),
            item_type: ItemTypeDto::Inventory,
            default_purchase_price: "10.50".parse().unwrap(),
            default_sales_price: "12.75".parse().unwrap(),
            track_inventory: true,
            is_active: true,
        };

        let json = serde_json::to_value(response).unwrap();

        assert_eq!(
            json,
            serde_json::json!({
                "publicId": "item_123",
                "sku": "ITEM-001",
                "barcode": "12345",
                "namePrimary": "Milk",
                "nameSecondary": "Full Cream",
                "itemType": "inventory",
                "defaultPurchasePrice": "10.50",
                "defaultSalesPrice": "12.75",
                "trackInventory": true,
                "isActive": true
            })
        );
    }
}
