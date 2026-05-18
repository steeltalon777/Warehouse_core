//! AI Tool contracts — allowlisted readonly tools for AI agents.
//!
//! These are the tool definitions that desktop MCP and other AI hosts
//! may expose. All tools enforce role/site scope and produce audit logs.
//!
//! Implementation: Level 7 of RUST_CORE_CAPABILITY_PLAN.md

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Describes an available tool for AI consumption.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value, // JSON Schema for function-calling
    pub requires_active_site: bool,
    pub is_mutating: bool,
    pub min_role: Option<String>,
}

/// Result envelope for a tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_name: String,
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub audit_id: Uuid,
}

/// Audit record for every tool invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolAuditEntry {
    pub audit_id: Uuid,
    pub tool_name: String,
    pub parameters: serde_json::Value,
    pub invoked_by: String,
    pub invoked_at: String,
    pub site_id: Option<i32>,
    pub success: bool,
    pub error_summary: Option<String>,
}

/// Approval envelope for mutating actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionApproval {
    pub audit_id: Uuid,
    pub tool_name: String,
    pub description: String,
    pub proposed_changes: serde_json::Value,
    pub status: ApprovalStatus,
    pub approved_by: Option<String>,
    pub approved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}
