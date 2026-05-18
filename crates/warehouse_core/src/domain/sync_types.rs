//! Sync protocol DTOs (Ping / Push / Pull / Bootstrap)

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// POST /ping request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingRequest {
    pub site_id: i32,
    pub device_id: Uuid,
    pub last_server_seq: i64,
    pub outbox_count: i32,
    pub client_time: String,
}

/// POST /ping response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResponse {
    pub server_time: String,
    pub server_seq_upto: i64,
    #[serde(default)]
    pub backoff_seconds: Option<f64>,
}

/// Event payload for push
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventPayload {
    pub doc_id: String,
    pub doc_type: String,
    pub comment: Option<String>,
    pub lines: Vec<EventLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventLine {
    pub item_id: i32,
    pub qty: serde_json::Value,
    #[serde(default)]
    pub batch: Option<String>,
}

/// Single event for push
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventIn {
    pub event_uuid: Uuid,
    pub event_type: String,
    pub event_datetime: String,
    pub schema_version: String,
    pub payload: EventPayload,
}

/// POST /push request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushRequest {
    pub site_id: i32,
    pub device_id: Uuid,
    pub batch_id: Uuid,
    pub events: Vec<EventIn>,
}

/// POST /push response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushResponse {
    pub accepted: Vec<Uuid>,
    pub duplicates: Vec<Uuid>,
    pub rejected: Vec<Uuid>,
    pub server_time: String,
    pub server_seq_upto: i64,
}

/// Pull event from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullEvent {
    pub server_seq: i64,
    pub event_uuid: Uuid,
    pub event_type: String,
    pub event_datetime: String,
    pub schema_version: String,
    pub payload: serde_json::Value,
    pub source_device_id: Uuid,
}

/// POST /pull request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub site_id: i32,
    pub device_id: Uuid,
    pub since_seq: i64,
    pub limit: Option<i32>,
}

/// POST /pull response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullResponse {
    pub events: Vec<PullEvent>,
    pub server_time: String,
    pub server_seq_upto: i64,
    pub next_since_seq: i64,
}

/// POST /bootstrap/sync response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapResponse {
    pub server_time: String,
    pub protocol_version: String,
    pub is_root: bool,
    pub root_user: Option<RootUserInfo>,
    pub root_role: Option<String>,
    pub device_id: Uuid,
    pub device_registered: bool,
    pub bootstrap_data: BootstrapData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootUserInfo {
    pub user_id: Uuid,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapData {
    pub available_sites: Vec<BootstrapSite>,
    pub protocol_version: String,
    #[serde(default)]
    pub settings: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapSite {
    pub site_id: i32,
    pub code: String,
    pub name: String,
}
