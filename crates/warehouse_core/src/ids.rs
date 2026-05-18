/// Core identifier types used across the warehouse domain.
use uuid::Uuid;

/// Bundles the core identity state for a client instance.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct CoreIds {
    /// Device UUID assigned by SyncServer during bootstrap
    pub device_id: Option<Uuid>,
    /// Active site ID
    pub site_id: Option<i32>,
    /// Authenticated user UUID
    pub user_id: Option<Uuid>,
}

impl CoreIds {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_bootstrapped(&self) -> bool {
        self.device_id.is_some()
    }
}
