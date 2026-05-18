use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    #[default]
    Waybill,
    AcceptanceCertificate,
    Act,
    Invoice,
    #[serde(untagged)]
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DocumentStatus {
    #[default]
    Draft,
    Finalized,
    Void,
    Superseded,
    #[serde(untagged)]
    Unknown(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentDto {
    pub id: uuid::Uuid,
    pub document_type: DocumentType,
    pub status: DocumentStatus,
    #[serde(default)]
    pub document_number: Option<String>,
    pub revision: i32,
    pub site_id: i32,
    #[serde(default)]
    pub template_name: Option<String>,
    #[serde(default)]
    pub template_version: Option<String>,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
    #[serde(default)]
    pub created_by_user_id: Option<uuid::Uuid>,
    pub created_at: String,
    #[serde(default)]
    pub finalized_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentGenerateRequest {
    pub operation_id: uuid::Uuid,
    pub document_type: DocumentType,
    #[serde(default)]
    pub template_name: Option<String>,
    #[serde(default)]
    pub auto_finalize: bool,
    #[serde(default)]
    pub language: String,
}
