use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::auth::ProfileService;
use crate::operations::OperationDraftService;
use crate::operations::outbox_service::OutboxService;
use crate::storage::cursor_store::CursorStore;
use crate::storage::repos::{
    SqliteAuthContextRepo, SqliteDraftRepo, SqliteErrorLogRepo, SqliteOutboxRepo,
};
use crate::storage::snapshot_writer::SnapshotWriter;
use crate::sync::bootstrap::BootstrapService;
use crate::sync::conflict::ConflictSummary;
use crate::sync::pull::PullSyncService;
use crate::syncserver::SyncServerClient;

/// Sync modes supported by the engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyncMode {
    Bootstrap,
    PullOnly,
    PushOnly,
    PushThenPull,
    Full,
}

/// Current phase of a sync run.
#[derive(Debug, Clone, PartialEq)]
pub enum SyncPhase {
    Idle,
    Ping,
    Push,
    Pull,
    Bootstrap,
    Complete,
    Error,
}

/// Progress event emitted during sync.
#[derive(Debug, Clone)]
pub struct SyncProgress {
    pub phase: SyncPhase,
    pub message: String,
    pub progress_pct: u8,
}

/// Result of a sync run.
#[derive(Debug, Clone, Default)]
pub struct SyncResult {
    pub success: bool,
    pub mode: Option<SyncMode>,
    pub push_accepted: i64,
    pub push_failed: i64,
    pub push_conflicts: i64,
    pub pull_items: usize,
    pub pull_errors: usize,
    pub bootstrap_ok: bool,
    pub conflicts: ConflictSummary,
    pub error: Option<String>,
}

/// Progress callback type.
pub type ProgressCallback = Box<dyn Fn(SyncProgress) + Send>;

/// SyncEngine orchestrates bootstrap, push, and pull phases.
pub struct SyncEngine {
    client: SyncServerClient,
    pool: sqlx::SqlitePool,
    profile: ProfileService,
    is_syncing: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    progress_cb: Option<ProgressCallback>,
}

impl SyncEngine {
    pub fn new(client: SyncServerClient, pool: sqlx::SqlitePool, profile: ProfileService) -> Self {
        Self {
            client,
            pool,
            profile,
            is_syncing: Arc::new(AtomicBool::new(false)),
            cancelled: Arc::new(AtomicBool::new(false)),
            progress_cb: None,
        }
    }

    pub fn set_progress_callback(&mut self, cb: ProgressCallback) {
        self.progress_cb = Some(cb);
    }

    pub fn set_cancel_flag(&mut self, flag: Arc<AtomicBool>) {
        self.cancelled = flag;
    }

    pub fn is_syncing(&self) -> bool {
        self.is_syncing.load(Ordering::SeqCst)
    }

    pub fn into_profile(self) -> ProfileService {
        self.profile
    }

    pub fn sync_lock(&self) -> Arc<AtomicBool> {
        self.is_syncing.clone()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    fn emit(&self, phase: SyncPhase, message: &str, pct: u8) {
        if let Some(ref cb) = self.progress_cb {
            cb(SyncProgress {
                phase,
                message: message.to_string(),
                progress_pct: pct,
            });
        }
    }

    pub async fn run(&mut self, mode: SyncMode) -> SyncResult {
        if self.is_syncing.swap(true, Ordering::SeqCst) {
            return SyncResult {
                error: Some("Sync already in progress".to_string()),
                ..Default::default()
            };
        }

        self.cancelled.store(false, Ordering::SeqCst);

        let result = self.run_inner(mode).await;

        self.is_syncing.store(false, Ordering::SeqCst);
        result
    }

    fn check_cancelled(&self, result: &mut SyncResult) -> bool {
        if self.cancelled.load(Ordering::SeqCst) {
            result.error = Some("Sync cancelled".to_string());
            self.emit(SyncPhase::Error, "Sync cancelled by user", 0);
            true
        } else {
            false
        }
    }

    async fn run_inner(&mut self, mode: SyncMode) -> SyncResult {
        let mut result = SyncResult {
            mode: Some(mode),
            ..Default::default()
        };

        // Phase: Ping (for non-bootstrap modes with device token)
        if mode != SyncMode::Bootstrap {
            if self.check_cancelled(&mut result) {
                return result;
            }
            self.emit(SyncPhase::Ping, "Pinging server...", 5);
            if self.profile.is_authenticated() {
                match self.client.health().await {
                    Ok(_) => self.emit(SyncPhase::Ping, "Server reachable", 10),
                    Err(e) => {
                        let msg = format!("Health check failed: {e}");
                        self.emit(SyncPhase::Error, &msg, 10);
                        result.error = Some(msg);
                        return result;
                    }
                }
            }
        }

        // Phase: Push
        if mode == SyncMode::PushOnly || mode == SyncMode::PushThenPull || mode == SyncMode::Full {
            if self.check_cancelled(&mut result) {
                return result;
            }
            self.emit(SyncPhase::Push, "Sending pending outbox events...", 20);
            let pool = self.pool.clone();
            let outbox_repo = SqliteOutboxRepo::new(pool.clone());
            let draft_repo = SqliteDraftRepo::new(pool);
            let draft_svc = OperationDraftService::new(draft_repo);
            let outbox_svc = OutboxService::new(outbox_repo, draft_svc);

            match outbox_svc.send_pending(&self.client, 10).await {
                Ok(sr) => {
                    result.push_accepted = sr.accepted;
                    result.push_failed = sr.failed;
                    result.push_conflicts = sr.conflicts;
                    self.emit(
                        SyncPhase::Push,
                        &format!(
                            "Push done: {} accepted, {} failed, {} conflicts",
                            sr.accepted, sr.failed, sr.conflicts
                        ),
                        40,
                    );
                }
                Err(e) => {
                    let msg = format!("Push failed: {e}");
                    self.emit(SyncPhase::Error, &msg, 40);
                    result.error = Some(msg);
                    return result;
                }
            }
        }

        // Phase: Bootstrap
        if mode == SyncMode::Bootstrap || mode == SyncMode::Full {
            if self.check_cancelled(&mut result) {
                return result;
            }
            self.emit(SyncPhase::Bootstrap, "Bootstrapping...", 45);
            let pool = self.pool.clone();
            let auth_repo = SqliteAuthContextRepo::new(pool.clone());
            let cursor_store = CursorStore::new(pool.clone());
            let writer = SnapshotWriter::new(pool);
            let mut bs =
                BootstrapService::new(self.client.clone(), auth_repo, cursor_store, writer);
            let existing = self.profile.current().ok();
            match bs.run_bootstrap(existing).await {
                Ok(br) => {
                    result.bootstrap_ok = br.success;
                    if br.success {
                        self.emit(SyncPhase::Bootstrap, "Bootstrap complete", 75);
                    } else {
                        self.emit(SyncPhase::Bootstrap, "Bootstrap had errors", 75);
                    }
                }
                Err(e) => {
                    let msg = format!("Bootstrap failed: {e}");
                    self.emit(SyncPhase::Error, &msg, 75);
                    result.error = Some(msg);
                    return result;
                }
            }
        }

        // Phase: Pull (skip if push-only or bootstrap-only)
        if mode == SyncMode::PullOnly || mode == SyncMode::PushThenPull || mode == SyncMode::Full {
            if self.check_cancelled(&mut result) {
                return result;
            }
            self.emit(SyncPhase::Pull, "Pulling data...", 55);
            let pool = self.pool.clone();
            let cursor_store = CursorStore::new(pool.clone());
            let writer = SnapshotWriter::new(pool.clone());
            let error_log = SqliteErrorLogRepo::new(pool);
            let mut ps = PullSyncService::new(
                self.client.clone(),
                std::mem::take(&mut self.profile),
                cursor_store,
                writer,
                error_log,
            );
            let summary = ps.pull_all().await;
            result.pull_items = summary.total_items;
            result.pull_errors = summary.errors_count;
            self.profile = ps.into_profile();

            // Propagate required-family failures to SyncResult
            let required_families = [
                "sites",
                "catalog_items",
                "catalog_categories",
                "catalog_units",
            ];
            let failed_required: Vec<String> = summary
                .families
                .iter()
                .filter(|f| !f.success && required_families.contains(&f.name.as_str()))
                .map(|f| f.name.clone())
                .collect();
            if !failed_required.is_empty() {
                let msg = format!(
                    "Pull failed for required families: {}",
                    failed_required.join(", ")
                );
                self.emit(SyncPhase::Error, &msg, 90);
                result.error = Some(msg);
            }

            self.emit(
                SyncPhase::Pull,
                &format!(
                    "Pull done: {} items, {} errors",
                    summary.total_items, summary.errors_count
                ),
                90,
            );
        }

        result.success = result.error.is_none();
        self.emit(SyncPhase::Complete, "Sync complete", 100);
        result
    }
}
