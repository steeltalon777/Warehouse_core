use serde::{Deserialize, Serialize};

/// Item from SyncServer /catalog/items
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemDto {
    pub id: i32,
    #[serde(default)]
    pub sku: Option<String>,
    pub name: String,
    pub category_id: i32,
    pub unit_id: i32,
    #[serde(default)]
    pub description: Option<String>,
    pub is_active: bool,
    #[serde(default)]
    pub hashtags: Option<Vec<String>>,
    pub updated_at: String,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub created_by_user_id: Option<String>,
    #[serde(default)]
    pub updated_by_user_id: Option<String>,
    #[serde(default)]
    pub created_by_user_name: Option<String>,
    #[serde(default)]
    pub updated_by_user_name: Option<String>,
}

/// Category from /catalog/categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryDto {
    pub id: i32,
    pub name: String,
    #[serde(default)]
    pub parent_id: Option<i32>,
    pub is_active: bool,
    pub updated_at: String,
    #[serde(default)]
    pub created_by_user_id: Option<String>,
    #[serde(default)]
    pub updated_by_user_id: Option<String>,
    #[serde(default)]
    pub created_by_user_name: Option<String>,
    #[serde(default)]
    pub updated_by_user_name: Option<String>,
}

/// Unit from /catalog/units
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitDto {
    pub id: i32,
    pub name: String,
    pub symbol: String,
    pub is_active: bool,
    pub updated_at: String,
    #[serde(default)]
    pub created_by_user_id: Option<String>,
    #[serde(default)]
    pub updated_by_user_id: Option<String>,
    #[serde(default)]
    pub created_by_user_name: Option<String>,
    #[serde(default)]
    pub updated_by_user_name: Option<String>,
}

/// CatalogSiteDto from /catalog/sites
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogSiteDto {
    pub site_id: i32,
    pub code: String,
    pub name: String,
    pub is_active: bool,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub permissions: Option<std::collections::HashMap<String, bool>>,
}

/// Category tree node (recursive)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryTreeNode {
    pub id: i32,
    pub name: String,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub parent_id: Option<i32>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub sort_order: Option<i32>,
    pub path: Vec<String>,
    #[serde(default)]
    pub children: Vec<CategoryTreeNode>,
}
