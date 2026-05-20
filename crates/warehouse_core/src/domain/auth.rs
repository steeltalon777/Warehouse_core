use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Auth context from GET /auth/context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContext {
    pub user: AuthContextUser,
    pub role: String,
    #[serde(rename = "is_root")]
    pub is_root: bool,
    #[serde(default)]
    pub default_site: Option<AuthSiteInfo>,
    #[serde(default)]
    pub available_sites: Vec<AuthSiteInfo>,
    pub device: Option<AuthDeviceInfo>,
    #[serde(default)]
    pub permissions_summary: Option<HashMap<String, bool>>,
}

/// User info within auth context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthContextUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub is_active: Option<bool>,
    pub role: String,
    #[serde(rename = "is_root")]
    pub is_root: bool,
    #[serde(default)]
    pub default_site_id: Option<i32>,
}

/// Device info within auth context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthDeviceInfo {
    #[serde(default)]
    pub id: i32,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub device_type: Option<String>,
    #[serde(default)]
    pub is_active: Option<bool>,
}

/// Site info within auth context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSiteInfo {
    pub site_id: i32,
    pub code: String,
    pub name: String,
    pub permissions: HashMap<String, bool>,
}

/// Response from GET /auth/sync-user (bootstrap)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncUserResponse {
    pub user_id: Uuid,
    pub user_token: Uuid,
    pub username: String,
    pub email: String,
    pub role: String,
    pub is_root: bool,
    pub default_site_id: Option<i32>,
}
