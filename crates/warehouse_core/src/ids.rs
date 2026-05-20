/// Core identifier types used across the warehouse domain.
/// Bundles the core identity state for a client instance.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CoreIds {
    /// Device ID assigned by SyncServer during bootstrap
    #[serde(default)]
    pub device_id: Option<i32>,
    /// Active site ID
    pub site_id: Option<i32>,
    /// Authenticated user UUID
    pub user_id: Option<uuid::Uuid>,
}

impl CoreIds {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_bootstrapped(&self) -> bool {
        self.device_id.is_some()
    }
}
