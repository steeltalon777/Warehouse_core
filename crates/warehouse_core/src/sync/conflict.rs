use serde::Serialize;
use std::collections::HashMap;

/// Types of conflicts that can occur during sync.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
    PermissionChanged,
    SiteAccessChanged,
    CatalogItemMissing,
    CatalogItemInactive,
    BalanceChanged,
    ServerValidationRejected,
    DuplicateOperation,
    ProtocolMismatch,
    UnknownServerRejection,
    Other(String),
}

impl std::fmt::Display for ConflictType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConflictType::PermissionChanged => write!(f, "permission_changed"),
            ConflictType::SiteAccessChanged => write!(f, "site_access_changed"),
            ConflictType::CatalogItemMissing => write!(f, "catalog_item_missing"),
            ConflictType::CatalogItemInactive => write!(f, "catalog_item_inactive"),
            ConflictType::BalanceChanged => write!(f, "balance_changed"),
            ConflictType::ServerValidationRejected => write!(f, "server_validation_rejected"),
            ConflictType::DuplicateOperation => write!(f, "duplicate_operation"),
            ConflictType::ProtocolMismatch => write!(f, "protocol_mismatch"),
            ConflictType::UnknownServerRejection => write!(f, "unknown_server_rejection"),
            ConflictType::Other(s) => write!(f, "{s}"),
        }
    }
}

/// A single conflict entry.
#[derive(Debug, Clone, Serialize)]
pub struct ConflictInfo {
    pub conflict_type: ConflictType,
    pub message: String,
    pub event_uuid: Option<String>,
    pub draft_id: Option<String>,
    pub resolved: bool,
}

/// Summary of all conflicts from a sync run.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ConflictSummary {
    pub total: usize,
    pub by_type: HashMap<String, usize>,
    pub unresolved: Vec<ConflictInfo>,
}

impl ConflictSummary {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, conflict: ConflictInfo) {
        let type_name = conflict.conflict_type.to_string();
        *self.by_type.entry(type_name).or_insert(0) += 1;
        if !conflict.resolved {
            self.unresolved.push(conflict);
        }
        self.total += 1;
    }

    pub fn is_empty(&self) -> bool {
        self.total == 0
    }
}
