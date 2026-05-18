use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    Root,
    ChiefStorekeeper,
    Storekeeper,
    #[default]
    Observer,
    #[serde(untagged)]
    Unknown(String),
}

// ── User ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCreate {
    pub username: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default = "default_true")]
    pub is_active: bool,
    #[serde(default)]
    pub is_root: bool,
    #[serde(default)]
    pub role: UserRole,
    #[serde(default)]
    pub default_site_id: Option<i32>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: uuid::Uuid,
    pub username: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub full_name: Option<String>,
    pub is_active: bool,
    pub is_root: bool,
    pub role: UserRole,
    #[serde(default)]
    pub default_site_id: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserWithTokenResponse {
    pub id: uuid::Uuid,
    pub username: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub full_name: Option<String>,
    pub is_active: bool,
    pub is_root: bool,
    pub role: UserRole,
    #[serde(default)]
    pub default_site_id: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
    pub user_token: uuid::Uuid,
}

// ── Device ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceResponse {
    pub device_id: i32,
    pub device_code: String,
    pub device_name: String,
    #[serde(default)]
    pub site_id: Option<i32>,
    pub is_active: bool,
    #[serde(default)]
    pub last_seen_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceWithTokenResponse {
    pub device_id: i32,
    pub device_code: String,
    pub device_name: String,
    #[serde(default)]
    pub site_id: Option<i32>,
    pub is_active: bool,
    #[serde(default)]
    pub last_seen_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub device_token: uuid::Uuid,
}

// ── Site ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminSiteCreate {
    pub code: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminSiteResponse {
    pub site_id: i32,
    pub code: String,
    pub name: String,
    pub is_active: bool,
    #[serde(default)]
    pub description: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

// ── UserAccessScope ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccessScopeResponse {
    pub id: i64,
    pub user_id: uuid::Uuid,
    pub site_id: i32,
    pub can_view: bool,
    pub can_operate: bool,
    pub can_manage_catalog: bool,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}
