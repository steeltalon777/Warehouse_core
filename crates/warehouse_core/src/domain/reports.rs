//! Reports DTOs

use serde::{Deserialize, Serialize};

/// Row from /reports/item-movement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemMovementRow {
    pub item_id: i32,
    pub item_name: String,
    pub item_sku: Option<String>,
    pub unit_symbol: String,
    pub operation_type: String,
    #[serde(deserialize_with = "crate::domain::serde_helpers::string_or_number")]
    pub operation_id: String,
    pub quantity: serde_json::Value,
    pub effective_at: String,
    pub site_id: i32,
    pub site_code: String,
}

/// Row from /reports/stock-summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockSummaryRow {
    #[serde(default)]
    pub item_id: i32,
    #[serde(alias = "display_name", alias = "resolved_item_name")]
    pub item_name: String,
    #[serde(default, alias = "sku")]
    pub item_sku: Option<String>,
    #[serde(default)]
    pub unit_symbol: String,
    pub site_id: i32,
    #[serde(alias = "site_name")]
    pub site_code: String,
    #[serde(alias = "total_quantity")]
    pub quantity: serde_json::Value,
    #[serde(alias = "last_balance_at")]
    pub last_operation_at: Option<String>,
}

/// Cached report result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedReport<T> {
    pub params_hash: String,
    pub refreshed_at: String,
    pub data: Vec<T>,
}
