use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Site info for the current user with permission flags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteDto {
    pub site_id: i32,
    pub code: String,
    pub name: String,
    pub is_active: bool,
    #[serde(default)]
    pub permissions: Option<HashMap<String, bool>>,
}
