use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TemporaryItemStatus {
    #[default]
    Active,
    ApprovedAsItem,
    MergedToItem,
    Deleted,
    #[serde(untagged)]
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryItemDto {
    pub id: i32,
    pub name: String,
    #[serde(default)]
    pub sku: Option<String>,
    #[serde(default)]
    pub category_id: Option<i32>,
    pub unit_id: i32,
    #[serde(default)]
    pub description: Option<String>,
    pub status: TemporaryItemStatus,
    #[serde(default)]
    pub resolved_item_id: Option<i32>,
    #[serde(default)]
    pub resolved_item_name: Option<String>,
    pub created_by_user_id: uuid::Uuid,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproveAsItemRequest {
    pub name: String,
    #[serde(default)]
    pub sku: Option<String>,
    #[serde(default)]
    pub category_id: Option<i32>,
    pub unit_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeToItemRequest {
    pub target_item_id: i32,
}
