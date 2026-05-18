use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecipientType {
    #[default]
    Person,
    Group,
    Department,
    Contractor,
    SystemRepo,
    #[serde(untagged)]
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipientCreate {
    pub name: String,
    pub recipient_type: RecipientType,
    #[serde(default)]
    pub contact_info: Option<String>,
    #[serde(default)]
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipientDto {
    pub id: i32,
    pub name: String,
    pub recipient_type: RecipientType,
    #[serde(default)]
    pub contact_info: Option<String>,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}
