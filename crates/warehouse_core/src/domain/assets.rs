use serde::{Deserialize, Serialize};

/// Row from /pending-acceptance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingAcceptanceRow {
    pub operation_id: i64,
    pub operation_line_id: i64,
    pub item_id: i32,
    pub item_name: String,
    #[serde(default)]
    pub item_sku: Option<String>,
    pub unit_symbol: String,
    pub qty: serde_json::Value,
    #[serde(default)]
    pub accepted_qty: Option<serde_json::Value>,
    #[serde(default)]
    pub lost_qty: Option<serde_json::Value>,
}

/// Row from /lost-assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LostAssetRow {
    pub operation_id: i64,
    pub operation_line_id: i64,
    pub item_id: i32,
    pub item_name: String,
    #[serde(default)]
    pub item_sku: Option<String>,
    pub unit_symbol: String,
    pub qty: serde_json::Value,
    pub lost_qty: serde_json::Value,
    pub is_resolved: bool,
}

/// Row from /issued-assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuedAssetRow {
    pub operation_id: i64,
    pub operation_line_id: i64,
    pub item_id: i32,
    pub item_name: String,
    #[serde(default)]
    pub item_sku: Option<String>,
    pub unit_symbol: String,
    pub qty: serde_json::Value,
    pub issued_to_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LostAssetResolveAction {
    FoundToDestination,
    ReturnToSource,
    WriteOff,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LostAssetResolveRequest {
    pub action: LostAssetResolveAction,
    pub resolved_qty: serde_json::Value,
}
