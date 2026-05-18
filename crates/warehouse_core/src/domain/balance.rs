use serde::{Deserialize, Serialize};

/// Balance row from /balances or /balances/by-site
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceRow {
    pub site_id: i32,
    pub site_code: String,
    pub item_id: i32,
    pub item_name: String,
    #[serde(default)]
    pub item_sku: Option<String>,
    pub unit_symbol: String,
    pub qty: serde_json::Value,
    pub updated_at: String,
    #[serde(default)]
    pub is_subject: Option<bool>,
}

/// Balance summary from /balances/summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSummaryRow {
    pub item_id: i32,
    pub item_name: String,
    pub total_qty: serde_json::Value,
    pub site_count: i32,
}
