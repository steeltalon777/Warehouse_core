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
    pub operation_id: i64,
    pub quantity: serde_json::Value,
    pub effective_at: String,
    pub site_id: i32,
    pub site_code: String,
}

/// Row from /reports/stock-summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockSummaryRow {
    pub item_id: i32,
    pub item_name: String,
    pub item_sku: Option<String>,
    pub unit_symbol: String,
    pub site_id: i32,
    pub site_code: String,
    pub quantity: serde_json::Value,
    pub last_operation_at: Option<String>,
}

/// Cached report result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedReport<T> {
    pub params_hash: String,
    pub refreshed_at: String,
    pub data: Vec<T>,
}
