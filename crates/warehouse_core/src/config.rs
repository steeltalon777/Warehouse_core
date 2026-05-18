use std::path::PathBuf;

/// Core configuration for the offline-first runtime.
///
/// All paths should be absolute. The config is typically loaded from
/// a JSON file in the user's profile directory.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CoreConfig {
    /// Base URL of the SyncServer (e.g. "https://sync.example.com")
    pub server_base_url: String,

    /// Path to the local SQLite database file
    pub database_path: PathBuf,

    /// Client display name for server identification
    pub client_name: String,

    /// Client version string
    #[serde(default = "default_client_version")]
    pub client_version: String,

    /// Registered device ID UUID (set after bootstrap)
    #[serde(default)]
    pub device_id: Option<uuid::Uuid>,

    /// Active site ID (set after login/site selection)
    #[serde(default)]
    pub site_id: Option<i32>,

    /// HTTP request timeout in seconds
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,

    /// Maximum retry attempts for network requests
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Retry backoff base in seconds
    #[serde(default = "default_retry_backoff_seconds")]
    pub retry_backoff_seconds: u64,
}

fn default_client_version() -> String {
    crate::CLIENT_VERSION.to_string()
}

fn default_timeout_seconds() -> u64 {
    30
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_backoff_seconds() -> u64 {
    2
}

impl Default for CoreConfig {
    fn default() -> Self {
        let db_path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("warehouse_client_core")
            .join("warehouse.db");

        Self {
            server_base_url: "http://localhost:8000".to_string(),
            database_path: db_path,
            client_name: crate::CLIENT_NAME.to_string(),
            client_version: default_client_version(),
            device_id: None,
            site_id: None,
            timeout_seconds: default_timeout_seconds(),
            max_retries: default_max_retries(),
            retry_backoff_seconds: default_retry_backoff_seconds(),
        }
    }
}

impl CoreConfig {
    /// Create a minimal config suitable for testing
    pub fn for_testing(db_path: PathBuf) -> Self {
        Self {
            database_path: db_path,
            server_base_url: "http://localhost:8000".to_string(),
            ..Default::default()
        }
    }
}
