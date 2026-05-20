use serde::{Deserialize, Serialize};

/// Row from /pending-acceptance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingAcceptanceRow {
    #[serde(deserialize_with = "crate::domain::serde_helpers::string_or_number")]
    pub operation_id: String,
    #[serde(deserialize_with = "crate::domain::serde_helpers::string_or_number")]
    pub operation_line_id: String,
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
    #[serde(deserialize_with = "crate::domain::serde_helpers::string_or_number")]
    pub operation_id: String,
    #[serde(deserialize_with = "crate::domain::serde_helpers::string_or_number")]
    pub operation_line_id: String,
    #[serde(default)]
    pub item_id: i32,
    #[serde(default, alias = "display_name", alias = "resolved_item_name")]
    pub item_name: String,
    #[serde(default, alias = "sku")]
    pub item_sku: Option<String>,
    #[serde(default)]
    pub unit_symbol: String,
    pub qty: serde_json::Value,
    #[serde(default)]
    pub lost_qty: serde_json::Value,
    #[serde(default)]
    pub is_resolved: bool,
}

/// Row from /issued-assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuedAssetRow {
    #[serde(default)]
    pub operation_id: String,
    #[serde(default)]
    pub operation_line_id: String,
    #[serde(default)]
    pub item_id: i32,
    #[serde(default, alias = "display_name", alias = "resolved_item_name")]
    pub item_name: String,
    #[serde(default, alias = "sku")]
    pub item_sku: Option<String>,
    #[serde(default)]
    pub unit_symbol: String,
    pub qty: serde_json::Value,
    #[serde(alias = "recipient_name")]
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
