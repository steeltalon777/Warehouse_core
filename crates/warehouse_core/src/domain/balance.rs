use serde::{Deserialize, Serialize};

/// Balance row from /balances or /balances/by-site
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceRow {
    pub site_id: i32,
    #[serde(alias = "site_name")]
    pub site_code: String,
    #[serde(default)]
    pub inventory_subject_id: i32,
    #[serde(default)]
    pub subject_type: String,
    #[serde(default)]
    pub item_id: i32,
    #[serde(default)]
    pub temporary_item_id: Option<i32>,
    #[serde(default, alias = "display_name", alias = "resolved_item_name")]
    pub item_name: String,
    #[serde(default, alias = "sku")]
    pub item_sku: Option<String>,
    #[serde(default)]
    pub unit_symbol: String,
    pub qty: serde_json::Value,
    pub updated_at: String,
    #[serde(default)]
    pub is_subject: Option<bool>,
}

/// Balance summary from /balances/summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSummaryRow {
    pub accessible_sites_count: i32,
    pub summary: BalanceSummaryData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceSummaryData {
    pub rows_count: i32,
    pub sites_count: i32,
    pub total_quantity: f64,
}
