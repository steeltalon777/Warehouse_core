//! Sync engine module.
//!
//! Contains:
//! - BootstrapService: first-run protocol check, identity, initial working set
//! - PullSyncService: incremental refresh of all data families
//! - SyncEngine: orchestrator that coordinates bootstrap/push/pull
//! - ConflictInfo, ConflictSummary: conflict types and collection
//! - SyncRunSummary, FamilyResult: pull run results

pub mod bootstrap;
pub mod conflict;
pub mod engine;
pub mod pull;

pub use bootstrap::{BootstrapResult, BootstrapService};
pub use conflict::{ConflictInfo, ConflictSummary, ConflictType};
pub use engine::{ProgressCallback, SyncEngine, SyncMode, SyncPhase, SyncProgress, SyncResult};
pub use pull::{FamilyResult, PullSyncService, SyncRunSummary};
