use sea_orm::entity::prelude::Decimal;
use serde::Deserialize;
use crate::entity::item::item_entity::ItemType;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemTypeDto {
    Inventory,
    Service,
    NonInventory,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_item_request_deserializes_item_type_values() {
        let request: CreateItemRequestDto = serde_json::from_value(serde_json::json!({
            "sku": "ITEM-001",
            "barcode": "12345",
            "name_primary": "Milk",
            "name_secondary": "Full Cream",
            "description": "Shelf item",
            "item_type": "non_inventory",
            "base_unit_public_id": "u_base",
            "purchase_unit_public_id": "u_purchase",
            "sales_unit_public_id": "u_sales",
            "default_purchase_price": "10.50",
            "default_sales_price": "12.75",
            "track_inventory": true,
            "allow_negative_stock": false,
            "reorder_level": "4.00",
            "is_active": true
        }))
        .unwrap();

        assert_eq!(request.item_type, ItemTypeDto::NonInventory);
        assert_eq!(request.base_unit_public_id, "u_base");
    }
}
