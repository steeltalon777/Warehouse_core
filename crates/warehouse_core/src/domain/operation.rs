use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OperationType {
    #[default]
    Receive,
    Expense,
    WriteOff,
    Move,
    Adjustment,
    Issue,
    IssueReturn,
    #[serde(untagged)]
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OperationStatus {
    Draft,
    Submitted,
    Cancelled,
    #[serde(untagged)]
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AcceptanceState {
    NotRequired,
    Pending,
    InProgress,
    Resolved,
    #[serde(untagged)]
    Unknown(String),
}

// ── Line create ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLineCreate {
    pub item_id: i32,
    pub qty: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub batch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryItemInlineCreate {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sku: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category_id: Option<i32>,
    pub unit_id: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LineItemOrTemp {
    ItemId { item_id: i32 },
    TemporaryItem(TemporaryItemInlineCreate),
}

// ── Operation create / update ────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationCreate {
    pub operation_type: OperationType,
    pub site_id: i32,
    pub lines: Vec<OperationLineCreate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_site_id: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination_site_id: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipient_id: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issued_to_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationUpdate {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_type: Option<OperationType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site_id: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lines: Option<Vec<OperationLineCreate>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_site_id: Option<Option<i32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination_site_id: Option<Option<i32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipient_id: Option<Option<i32>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issued_to_name: Option<Option<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<Option<String>>,
}

// ── Operation response ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLineResponse {
    pub id: i64,
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
    #[serde(default)]
    pub batch: Option<String>,
    #[serde(default)]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResponse {
    pub id: i64,
    pub operation_type: OperationType,
    pub status: OperationStatus,
    pub site_id: i32,
    pub site_code: String,
    pub lines: Vec<OperationLineResponse>,
    pub created_by_user_id: uuid::Uuid,
    pub created_by_user_name: String,
    #[serde(default)]
    pub effective_at: Option<String>,
    #[serde(default)]
    pub source_site_id: Option<i32>,
    #[serde(default)]
    pub destination_site_id: Option<i32>,
    #[serde(default)]
    pub recipient_id: Option<i32>,
    #[serde(default)]
    pub recipient_name: Option<String>,
    #[serde(default)]
    pub issued_to_name: Option<String>,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub acceptance_state: Option<AcceptanceState>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub submitted_at: Option<String>,
    #[serde(default)]
    pub cancelled_at: Option<String>,
}

// ── Accept lines ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptLineRequest {
    pub line_id: i64,
    pub accepted_qty: serde_json::Value,
    #[serde(default)]
    pub lost_qty: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptLinesRequest {
    pub lines: Vec<AcceptLineRequest>,
}

// ── Effective date ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetEffectiveAtRequest {
    pub effective_at: String,
}

// ── Operation history / list item ────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationListItem {
    pub id: i64,
    pub operation_type: OperationType,
    pub status: OperationStatus,
    pub site_id: i32,
    pub site_code: String,
    pub line_count: i32,
    pub total_qty: serde_json::Value,
    pub created_by_user_name: String,
    pub created_at: String,
    pub updated_at: String,
}

// ── Local draft model (separate from server DTO) ─────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationDraftLine {
    pub line_id: uuid::Uuid,
    pub item_id: Option<i32>,
    pub temporary_item: Option<TemporaryItemInlineCreate>,
    pub qty: serde_json::Value,
    pub batch: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationDraft {
    pub draft_id: uuid::Uuid,
    pub operation_type: OperationType,
    pub site_id: Option<i32>,
    pub lines: Vec<OperationDraftLine>,
    pub effective_at: Option<String>,
    pub source_site_id: Option<i32>,
    pub destination_site_id: Option<i32>,
    pub recipient_id: Option<i32>,
    pub issued_to_name: Option<String>,
    pub comment: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}
