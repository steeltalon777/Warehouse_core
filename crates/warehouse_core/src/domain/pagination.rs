use serde::{Deserialize, Serialize};

/// Standard paginated request parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRequest {
    pub page: u32,
    pub page_size: u32,
}

impl Default for PageRequest {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 100,
        }
    }
}

/// Standard paginated response envelope (SyncServer list endpoints)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    #[serde(alias = "total")]
    pub total_count: u64,
    #[serde(default, alias = "offset")]
    pub page: u32,
    #[serde(default, alias = "limit")]
    pub page_size: u32,
}

/// SyncServer `/catalog/items` uses next_updated_after cursor
/// Note: categories endpoint returns `categories` key, units returns `units` key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorResponse<T> {
    #[serde(alias = "categories", alias = "units")]
    pub items: Vec<T>,
    pub server_time: String,
    pub next_updated_after: Option<String>,
}
