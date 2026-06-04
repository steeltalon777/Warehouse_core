use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueObjectDto {
    pub id: i32,
    pub display_name: String,
    pub object_type: String,
    #[serde(default)]
    pub code: Option<String>,
    pub normalized_key: String,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub category_id: Option<i32>,
    pub is_active: bool,
    #[serde(default)]
    pub merged_into_id: Option<i32>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub deleted_at: Option<String>,
    #[serde(default)]
    pub deleted_by_user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueObjectCreate {
    pub display_name: String,
    pub object_type: String,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub category_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueObjectUpdate {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category_id: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueObjectMerge {
    pub source_id: i32,
    pub target_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueObjectListResponse {
    pub items: Vec<IssueObjectDto>,
    pub total_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueObjectCategoryDto {
    pub id: i32,
    pub name: String,
    pub normalized_key: String,
    #[serde(default)]
    pub parent_id: Option<i32>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub deleted_at: Option<String>,
    #[serde(default)]
    pub deleted_by_user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueObjectCategoryCreate {
    pub name: String,
    #[serde(default)]
    pub parent_id: Option<i32>,
    #[serde(default)]
    pub sort_order: Option<i32>,
    #[serde(default)]
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueObjectCategoryUpdate {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueObjectTreeDto {
    pub id: i32,
    pub r#type: String,
    pub name: String,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub category_id: Option<i32>,
    pub children: Vec<IssueObjectTreeDto>,
}
