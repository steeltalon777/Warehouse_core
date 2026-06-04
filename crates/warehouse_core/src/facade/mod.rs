use crate::auth::token_provider::{NullTokenProvider, TokenProvider};
use crate::auth::{Profile, ProfileService};
use crate::config::CoreConfig;
use crate::domain::assets::{
    IssuedAssetRow, LostAssetResolveRequest, LostAssetRow, PendingAcceptanceRow,
};
use crate::domain::auth::AuthSiteInfo;
use crate::domain::balance::{BalanceRow, BalanceSummaryRow};
use crate::domain::catalog::{CategoryDto, CategoryTreeNode, ItemDto, UnitDto};
use crate::domain::documents::{DocumentDto, DocumentGenerateRequest};
use crate::domain::issue_objects::{
    IssueObjectCategoryCreate, IssueObjectCategoryDto, IssueObjectCategoryUpdate,
    IssueObjectCreate, IssueObjectDto, IssueObjectListResponse, IssueObjectMerge,
    IssueObjectTreeDto, IssueObjectUpdate,
};
use crate::domain::operation::TemporaryItemInlineCreate;
use crate::domain::operation::{
    AcceptLinesRequest, OperationDraft, OperationListItem, OperationType,
};
use crate::domain::pagination::PaginatedResponse;
use crate::domain::recipient::RecipientDto;
use crate::domain::temporary_items::TemporaryItemDto;
use crate::error::CoreResult;
use crate::storage::Database;
use crate::storage::cursor_store::CursorStore;
use crate::storage::repos::{
    AssetsRepo, BalanceRepo, CatalogRepo, DocumentRepo, OutboxRepo, RecipientRepo, RepoBag,
    SqliteAuthContextRepo, SqliteErrorLogRepo, SqliteSyncRunRepo, TemporaryItemRepo,
};
use crate::storage::snapshot_writer::SnapshotWriter;
use crate::sync::{
    BootstrapService, ConflictSummary, PullSyncService, SyncEngine, SyncMode, SyncResult,
    SyncRunSummary,
};
use crate::syncserver::SyncServerClient;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone)]
pub struct DiagnosticsInfo {
    pub config: String,
    pub db_path: String,
    pub profile_exists: bool,
    pub profile_user: Option<String>,
    pub profile_role: Option<String>,
    pub active_site: Option<i32>,
    pub outbox_pending: i64,
    pub version: String,
}

pub struct CoreHandle {
    config: CoreConfig,
    db: Database,
    client: Option<SyncServerClient>,
    profile: ProfileService,
    token_provider: Box<dyn TokenProvider>,
    last_sync_result: Option<SyncResult>,
    sync_lock: Arc<AtomicBool>,
    cancel_requested: Arc<AtomicBool>,
}

impl CoreHandle {
    pub async fn open(config: CoreConfig) -> CoreResult<Self> {
        let db_path = config.database_path.to_string_lossy().to_string();
        let db = Database::open(&db_path, true).await?;
        Ok(Self {
            config,
            db,
            client: None,
            profile: ProfileService::new(),
            token_provider: Box::new(NullTokenProvider),
            last_sync_result: None,
            sync_lock: Arc::new(AtomicBool::new(false)),
            cancel_requested: Arc::new(AtomicBool::new(false)),
        })
    }

    pub async fn open_readonly(db_path: &str) -> CoreResult<Self> {
        let db = Database::open(db_path, false).await?;
        Ok(Self {
            config: CoreConfig::for_testing(std::path::PathBuf::from(db_path)),
            db,
            client: None,
            profile: ProfileService::new(),
            token_provider: Box::new(NullTokenProvider),
            last_sync_result: None,
            sync_lock: Arc::new(AtomicBool::new(false)),
            cancel_requested: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn close(&mut self) {
        self.client = None;
    }

    pub fn set_token_provider(&mut self, provider: Box<dyn TokenProvider>) {
        self.token_provider = provider;
    }

    pub fn config(&self) -> &CoreConfig {
        &self.config
    }

    pub fn db(&self) -> &Database {
        &self.db
    }

    pub fn client(&mut self) -> CoreResult<&SyncServerClient> {
        if self.client.is_none() {
            let mut c = SyncServerClient::new(&self.config)?;
            if let Some(token) = self.token_provider.user_token() {
                c.set_user_token(token);
            }
            if let Some(token) = self.token_provider.device_token() {
                c.set_device_token(token);
            }
            self.client = Some(c);
        }
        Ok(self.client.as_ref().unwrap())
    }

    pub fn profile(&self) -> &ProfileService {
        &self.profile
    }

    pub fn profile_mut(&mut self) -> &mut ProfileService {
        &mut self.profile
    }

    // ── Lifecycle ──────────────────────────────────────────────

    pub fn profile_status(&self) -> CoreResult<ProfileStatus> {
        let p = self.profile.current()?;
        Ok(ProfileStatus {
            user_id: p.user_id,
            user_name: p.user_name.clone(),
            user_email: p.user_email.clone(),
            role: p.role.clone(),
            is_root: p.is_root,
            device_id: p.device_id,
            device_registered: p.device_registered,
            active_site_id: p.active_site_id,
            protocol_version: p.protocol_version.clone(),
            refreshed_at: p.refreshed_at.clone(),
        })
    }

    pub fn health_local(&self) -> CoreResult<String> {
        let _ = self.db.pool();
        Ok("OK".to_string())
    }

    pub async fn diagnostics(&self) -> CoreResult<DiagnosticsInfo> {
        let repo = RepoBag::new(self.db.pool().clone());
        let profile_exists = self.profile.is_authenticated();
        let (profile_user, profile_role) = if profile_exists {
            let p = self.profile.current().ok();
            (p.map(|x| x.user_name.clone()), p.map(|x| x.role.clone()))
        } else {
            (None, None)
        };
        let outbox_pending = repo.outbox.count_pending().await.unwrap_or(0);
        Ok(DiagnosticsInfo {
            config: serde_json::to_string(&self.config).unwrap_or_default(),
            db_path: self.db.path().to_string(),
            profile_exists,
            profile_user,
            profile_role,
            active_site: self.profile.current().ok().and_then(|p| p.active_site_id),
            outbox_pending,
            version: crate::CLIENT_VERSION.to_string(),
        })
    }

    // ── Auth/Site ─────────────────────────────────────────────

    pub async fn load_profile(&mut self) -> CoreResult<()> {
        let repo = SqliteAuthContextRepo::new(self.db.pool().clone());
        self.profile.load(&repo).await
    }

    pub async fn refresh_identity(&mut self) -> CoreResult<()> {
        let ctx = {
            let client = self.client()?;
            client.auth_context().await?
        };
        let pool = self.db.pool().clone();
        let repo = SqliteAuthContextRepo::new(pool);
        self.profile.refresh_from_auth_context(ctx, &repo).await
    }

    pub fn get_auth_context(&self) -> CoreResult<AuthContextDto> {
        let p = self.profile.current()?;
        Ok(AuthContextDto {
            user_id: p.user_id,
            user_name: p.user_name.clone(),
            user_email: p.user_email.clone(),
            role: p.role.clone(),
            is_root: p.is_root,
            available_sites: p.available_sites.clone(),
            device_id: p.device_id,
            device_registered: p.device_registered,
        })
    }

    pub fn list_available_sites(&self) -> CoreResult<Vec<AuthSiteInfo>> {
        self.profile.sites().map(|s| s.to_vec())
    }

    pub async fn set_active_site(&mut self, site_id: i32) -> CoreResult<()> {
        let repo = SqliteAuthContextRepo::new(self.db.pool().clone());
        self.profile.set_active_site(site_id, &repo).await
    }

    pub fn get_active_site(&self) -> CoreResult<Option<i32>> {
        Ok(self.profile.current()?.active_site_id)
    }

    pub async fn clear_active_site(&mut self) -> CoreResult<()> {
        let repo = SqliteAuthContextRepo::new(self.db.pool().clone());
        self.profile.clear_active_site(&repo).await
    }

    pub async fn logout(&mut self) -> CoreResult<()> {
        let repo = SqliteAuthContextRepo::new(self.db.pool().clone());
        self.profile.logout(&repo).await?;
        self.client = None;
        Ok(())
    }

    // ── Connectivity ──────────────────────────────────────────

    pub async fn health_remote(&mut self) -> CoreResult<bool> {
        match self.client()?.health().await {
            Ok(s) => Ok(s.status == "ok" || s.status == "healthy"),
            Err(_) => Ok(false),
        }
    }

    pub async fn ready_remote(&mut self) -> CoreResult<String> {
        let ready = self.client()?.ready().await?;
        Ok(ready.status)
    }

    // ── Sync ──────────────────────────────────────────────────

    pub async fn bootstrap_device(&mut self) -> CoreResult<Profile> {
        let bootstrap = self.client()?.bootstrap_sync().await?;

        let (user_id, user_email) = bootstrap
            .root_user
            .as_ref()
            .map(|u| (u.user_id, u.email.clone()))
            .unwrap_or_default();
        let profile = Profile {
            user_id,
            user_name: String::new(),
            user_email,
            role: bootstrap.root_role.unwrap_or_else(|| "observer".into()),
            is_root: bootstrap.is_root,
            available_sites: bootstrap
                .bootstrap_data
                .available_sites
                .into_iter()
                .map(|s| crate::domain::auth::AuthSiteInfo {
                    site_id: s.site_id,
                    code: s.code,
                    name: s.name,
                    permissions: std::collections::HashMap::new(),
                })
                .collect(),
            device_id: bootstrap.device_id,
            device_registered: bootstrap.device_registered,
            active_site_id: None,
            protocol_version: bootstrap.protocol_version,
            refreshed_at: crate::time::Timestamp::now_utc().to_string(),
        };

        let repo = SqliteAuthContextRepo::new(self.db.pool().clone());
        profile.save(&repo).await?;
        self.profile = ProfileService::new();
        self.profile.load(&repo).await?;
        Ok(profile)
    }

    pub async fn bootstrap(&mut self) -> CoreResult<crate::sync::BootstrapResult> {
        let client = self.client()?.clone();
        let pool = self.db.pool().clone();
        let auth_repo = SqliteAuthContextRepo::new(pool.clone());
        let cursor_store = CursorStore::new(pool.clone());
        let writer = SnapshotWriter::new(pool);
        let mut bs = BootstrapService::new(client, auth_repo, cursor_store, writer);
        let existing = self.profile.current().ok();
        let result = bs.run_bootstrap(existing).await?;
        if let Some(ref profile) = result.profile {
            let repo = SqliteAuthContextRepo::new(self.db.pool().clone());
            profile.save(&repo).await?;
            self.profile = ProfileService::new();
            self.profile.load(&repo).await?;
        }
        Ok(result)
    }

    pub async fn pull_once(&mut self) -> SyncRunSummary {
        let client = match self.client() {
            Ok(c) => c.clone(),
            Err(_) => return SyncRunSummary::new(),
        };
        let pool = self.db.pool().clone();
        let cursor_store = CursorStore::new(pool.clone());
        let writer = SnapshotWriter::new(pool.clone());
        let error_log = SqliteErrorLogRepo::new(pool);
        let mut ps = PullSyncService::new(
            client,
            std::mem::take(&mut self.profile),
            cursor_store,
            writer,
            error_log,
        );
        let summary = ps.pull_all().await;
        self.profile = ps.into_profile();
        summary
    }

    pub async fn get_sync_status(&self) -> CoreResult<SyncStatusInfo> {
        let repo = RepoBag::new(self.db.pool().clone());
        let outbox_pending = repo.outbox.count_pending().await?;
        let cursor_store = CursorStore::new(self.db.pool().clone());
        let cursors = cursor_store.all_cursors().await?;
        Ok(SyncStatusInfo {
            is_authenticated: self.profile.is_authenticated(),
            active_site_id: self.profile.current().ok().and_then(|p| p.active_site_id),
            outbox_pending,
            cursor_count: cursors.len() as i32,
        })
    }

    pub async fn list_sync_runs(&self) -> CoreResult<Vec<SyncRunSummary>> {
        let repo = SqliteSyncRunRepo::new(self.db.pool().clone());
        repo.list_sync_runs(20).await
    }

    pub fn cancel_sync(&self) {
        self.cancel_requested.store(true, Ordering::Release);
        self.sync_lock.store(false, Ordering::Release);
    }

    // ── Catalog ───────────────────────────────────────────────

    pub async fn search_items(&self, query: &str) -> CoreResult<Vec<ItemDto>> {
        let repo = RepoBag::new(self.db.pool().clone());
        repo.catalog.search_items(query).await
    }

    pub async fn get_item(&self, id: i32) -> CoreResult<Option<ItemDto>> {
        let repo = RepoBag::new(self.db.pool().clone());
        repo.catalog.get_item(id).await
    }

    pub async fn list_units(&self) -> CoreResult<Vec<UnitDto>> {
        let repo = RepoBag::new(self.db.pool().clone());
        repo.catalog.all_units().await
    }

    pub async fn list_categories(&self) -> CoreResult<Vec<CategoryDto>> {
        let repo = RepoBag::new(self.db.pool().clone());
        repo.catalog.all_categories().await
    }

    pub async fn get_category_tree(&self) -> CoreResult<Vec<CategoryTreeNode>> {
        let cats = self.list_categories().await?;
        Ok(build_category_tree(&cats))
    }

    pub async fn get_parent_chain(&self, category_id: i32) -> CoreResult<Vec<CategoryDto>> {
        let cats = self.list_categories().await?;
        let mut chain = Vec::new();
        let mut current_id = Some(category_id);
        while let Some(cid) = current_id {
            if let Some(cat) = cats.iter().find(|c| c.id == cid) {
                chain.push(cat.clone());
                current_id = cat.parent_id;
            } else {
                break;
            }
        }
        chain.reverse();
        Ok(chain)
    }

    // ── Balances / Assets ─────────────────────────────────────

    pub async fn list_balances(&self, site_id: i32) -> CoreResult<Vec<BalanceRow>> {
        let repo = RepoBag::new(self.db.pool().clone());
        repo.balances.get_by_site(site_id).await
    }

    pub async fn get_balance_by_subject(&self, item_id: i32) -> CoreResult<Vec<BalanceRow>> {
        let repo = RepoBag::new(self.db.pool().clone());
        repo.balances.get_by_item(item_id).await
    }

    pub async fn get_balance_summary(&mut self) -> CoreResult<BalanceSummaryRow> {
        self.client()?.balances_summary(1, 200).await
    }

    pub async fn list_pending_acceptance(&self) -> CoreResult<Vec<PendingAcceptanceRow>> {
        let repo = RepoBag::new(self.db.pool().clone());
        repo.assets.get_pending_acceptance().await
    }

    pub async fn list_lost_assets(&self) -> CoreResult<Vec<LostAssetRow>> {
        let repo = RepoBag::new(self.db.pool().clone());
        repo.assets.get_lost_assets().await
    }

    pub async fn list_issued_assets(&self) -> CoreResult<Vec<IssuedAssetRow>> {
        let repo = RepoBag::new(self.db.pool().clone());
        repo.assets.get_issued_assets().await
    }

    // ── Operations Read ───────────────────────────────────────

    pub async fn list_operations(
        &mut self,
        site_id: i32,
        page: u32,
        page_size: u32,
    ) -> CoreResult<PaginatedResponse<OperationListItem>> {
        let sid = site_id.to_string();
        let mut filters = std::collections::HashMap::new();
        filters.insert("site_id", sid.as_str());
        self.client()?
            .operations_list(page, page_size.min(100), Some(filters))
            .await
    }

    pub async fn get_operation(
        &mut self,
        operation_id: &str,
    ) -> CoreResult<crate::domain::operation::OperationResponse> {
        self.client()?.operations_get(operation_id).await
    }

    // ── Recipients ────────────────────────────────────────────

    pub async fn search_recipients(&self, query: &str) -> CoreResult<Vec<RecipientDto>> {
        let repo = RepoBag::new(self.db.pool().clone());
        repo.recipients.search(query).await
    }

    pub async fn get_recipient(&self, id: i32) -> CoreResult<Option<RecipientDto>> {
        let repo = RepoBag::new(self.db.pool().clone());
        repo.recipients.get(id).await
    }

    // ── Temporary Items ───────────────────────────────────────

    pub async fn list_temporary_items(&self) -> CoreResult<Vec<TemporaryItemDto>> {
        let repo = RepoBag::new(self.db.pool().clone());
        repo.temporary_items.list_active().await
    }

    pub async fn get_temporary_item(&self, id: i32) -> CoreResult<Option<TemporaryItemDto>> {
        let repo = RepoBag::new(self.db.pool().clone());
        repo.temporary_items.get(id).await
    }

    pub async fn list_temporary_item_operations(
        &mut self,
        temp_id: i32,
    ) -> CoreResult<PaginatedResponse<OperationListItem>> {
        self.client()?
            .temporary_items_operations(temp_id, 1, 50)
            .await
    }

    // ── Documents / Reports ───────────────────────────────────

    pub async fn list_documents(&self, site_id: i32) -> CoreResult<Vec<DocumentDto>> {
        let repo = RepoBag::new(self.db.pool().clone());
        repo.documents.list_by_site(site_id).await
    }

    pub async fn get_document(&self, id: &str) -> CoreResult<Option<DocumentDto>> {
        let repo = RepoBag::new(self.db.pool().clone());
        repo.documents.get(id).await
    }

    pub async fn list_operation_documents(
        &mut self,
        operation_id: uuid::Uuid,
    ) -> CoreResult<Vec<DocumentDto>> {
        self.client()?.documents_for_operation(operation_id).await
    }

    pub async fn run_stock_summary(
        &mut self,
    ) -> CoreResult<
        crate::domain::pagination::PaginatedResponse<crate::domain::reports::StockSummaryRow>,
    > {
        self.client()?.reports_stock_summary(1, 200).await
    }

    pub async fn run_item_movement(
        &mut self,
    ) -> CoreResult<
        crate::domain::pagination::PaginatedResponse<crate::domain::reports::ItemMovementRow>,
    > {
        self.client()?
            .reports_item_movement(1, 200, None, None, None)
            .await
    }

    // ── Drafts ──────────────────────────────────────────────────

    pub async fn create_draft(
        &self,
        operation_type: OperationType,
        site_id: Option<i32>,
    ) -> CoreResult<OperationDraft> {
        let repo = RepoBag::new(self.db.pool().clone());
        let svc = crate::operations::OperationDraftService::new(repo.drafts);
        svc.create_draft(operation_type, site_id).await
    }

    pub async fn get_draft(&self, draft_id: &str) -> CoreResult<Option<OperationDraft>> {
        let repo = RepoBag::new(self.db.pool().clone());
        let svc = crate::operations::OperationDraftService::new(repo.drafts);
        svc.get_draft(draft_id).await
    }

    pub async fn list_drafts(&self) -> CoreResult<Vec<OperationDraft>> {
        let repo = RepoBag::new(self.db.pool().clone());
        let svc = crate::operations::OperationDraftService::new(repo.drafts);
        svc.list_drafts().await
    }

    pub async fn delete_draft(&self, draft_id: &str) -> CoreResult<()> {
        let repo = RepoBag::new(self.db.pool().clone());
        let svc = crate::operations::OperationDraftService::new(repo.drafts);
        svc.delete_draft(draft_id).await
    }

    pub async fn clone_draft(&self, draft_id: &str) -> CoreResult<Option<OperationDraft>> {
        let repo = RepoBag::new(self.db.pool().clone());
        let svc = crate::operations::OperationDraftService::new(repo.drafts);
        svc.clone_draft(draft_id).await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_draft_header(
        &self,
        draft_id: &str,
        operation_type: Option<OperationType>,
        site_id: Option<Option<i32>>,
        effective_at: Option<Option<String>>,
        source_site_id: Option<Option<i32>>,
        destination_site_id: Option<Option<i32>>,
        recipient_id: Option<Option<i32>>,
        issued_to_name: Option<Option<String>>,
        comment: Option<Option<String>>,
    ) -> CoreResult<Option<OperationDraft>> {
        let repo = RepoBag::new(self.db.pool().clone());
        let svc = crate::operations::OperationDraftService::new(repo.drafts);
        svc.update_header(
            draft_id,
            operation_type,
            site_id,
            effective_at,
            source_site_id,
            destination_site_id,
            recipient_id,
            issued_to_name,
            comment,
        )
        .await
    }

    pub async fn add_draft_item_line(
        &self,
        draft_id: &str,
        item_id: i32,
        qty: serde_json::Value,
        batch: Option<String>,
        comment: Option<String>,
    ) -> CoreResult<Option<OperationDraft>> {
        let repo = RepoBag::new(self.db.pool().clone());
        let svc = crate::operations::OperationDraftService::new(repo.drafts);
        svc.add_item_line(draft_id, item_id, qty, batch, comment)
            .await
    }

    pub async fn add_draft_temp_item_line(
        &self,
        draft_id: &str,
        temp_item: TemporaryItemInlineCreate,
        qty: serde_json::Value,
        batch: Option<String>,
        comment: Option<String>,
    ) -> CoreResult<Option<OperationDraft>> {
        let repo = RepoBag::new(self.db.pool().clone());
        let svc = crate::operations::OperationDraftService::new(repo.drafts);
        svc.add_temp_item_line(draft_id, temp_item, qty, batch, comment)
            .await
    }

    pub async fn update_draft_line(
        &self,
        draft_id: &str,
        line_id: &str,
        qty: Option<serde_json::Value>,
        batch: Option<Option<String>>,
        comment: Option<Option<String>>,
    ) -> CoreResult<Option<OperationDraft>> {
        let repo = RepoBag::new(self.db.pool().clone());
        let svc = crate::operations::OperationDraftService::new(repo.drafts);
        svc.update_line(draft_id, line_id, qty, batch, comment)
            .await
    }

    pub async fn delete_draft_line(
        &self,
        draft_id: &str,
        line_id: &str,
    ) -> CoreResult<Option<OperationDraft>> {
        let repo = RepoBag::new(self.db.pool().clone());
        let svc = crate::operations::OperationDraftService::new(repo.drafts);
        svc.delete_line(draft_id, line_id).await
    }

    pub async fn validate_draft(&self, draft_id: &str) -> Result<(), Vec<String>> {
        let repo = RepoBag::new(self.db.pool().clone());
        let svc = crate::operations::OperationDraftService::new(repo.drafts);
        let draft = match svc.get_draft(draft_id).await {
            Ok(Some(d)) => d,
            Ok(None) => return Err(vec![format!("Draft {draft_id} not found")]),
            Err(e) => return Err(vec![e.to_string()]),
        };
        match crate::operations::DraftValidator::validate(&draft) {
            Ok(()) => Ok(()),
            Err(errors) => Err(errors.into_iter().map(|e| e.to_string()).collect()),
        }
    }

    // ── Outbox ──────────────────────────────────────────────────

    pub async fn queue_draft_submit(&self, draft_id: &str) -> CoreResult<String> {
        let repo = RepoBag::new(self.db.pool().clone());
        let drafts = crate::operations::OperationDraftService::new(repo.drafts);
        let svc = crate::operations::OutboxService::new(repo.outbox, drafts);
        svc.queue_draft_submit(draft_id).await
    }

    pub async fn list_outbox_events(
        &self,
        site_id: Option<i32>,
        status: Option<&str>,
    ) -> CoreResult<Vec<crate::storage::repos::OutboxEvent>> {
        let repo = RepoBag::new(self.db.pool().clone());
        let drafts = crate::operations::OperationDraftService::new(repo.drafts);
        let svc = crate::operations::OutboxService::new(repo.outbox, drafts);
        svc.list_outbox_events(site_id, status).await
    }

    pub async fn get_outbox_event(
        &self,
        event_uuid: &str,
    ) -> CoreResult<Option<crate::storage::repos::OutboxEvent>> {
        let repo = RepoBag::new(self.db.pool().clone());
        let drafts = crate::operations::OperationDraftService::new(repo.drafts);
        let svc = crate::operations::OutboxService::new(repo.outbox, drafts);
        svc.get_outbox_event(event_uuid).await
    }

    pub async fn retry_outbox_event(&self, event_uuid: &str) -> CoreResult<()> {
        let repo = RepoBag::new(self.db.pool().clone());
        let drafts = crate::operations::OperationDraftService::new(repo.drafts);
        let svc = crate::operations::OutboxService::new(repo.outbox, drafts);
        svc.retry_event(event_uuid).await
    }

    pub async fn cancel_outbox_event(&self, event_uuid: &str) -> CoreResult<()> {
        let repo = RepoBag::new(self.db.pool().clone());
        let drafts = crate::operations::OperationDraftService::new(repo.drafts);
        let svc = crate::operations::OutboxService::new(repo.outbox, drafts);
        svc.cancel_event(event_uuid).await
    }

    pub async fn send_outbox(
        &mut self,
    ) -> CoreResult<crate::operations::outbox_service::SendResult> {
        let client = self.client()?.clone();
        let repo = RepoBag::new(self.db.pool().clone());
        let drafts = crate::operations::OperationDraftService::new(repo.drafts);
        let svc = crate::operations::OutboxService::new(repo.outbox, drafts);
        svc.send_pending(&client, 10).await
    }

    // ── Sync Engine ─────────────────────────────────────────────

    pub async fn sync_once(&mut self, mode: SyncMode) -> SyncResult {
        if self.sync_lock.swap(true, Ordering::AcqRel) {
            return SyncResult {
                error: Some("Sync already in progress".into()),
                mode: Some(mode),
                ..Default::default()
            };
        }
        let result = self.sync_once_inner(mode).await;
        self.sync_lock.store(false, Ordering::Release);
        result
    }

    pub fn is_syncing(&self) -> bool {
        self.sync_lock.load(Ordering::Acquire)
    }

    async fn sync_once_inner(&mut self, mode: SyncMode) -> SyncResult {
        let client = match self.client() {
            Ok(c) => c.clone(),
            Err(e) => {
                return SyncResult {
                    error: Some(e.to_string()),
                    mode: Some(mode),
                    ..Default::default()
                };
            }
        };
        self.cancel_requested.store(false, Ordering::Release);
        let pool = self.db.pool().clone();
        let profile = std::mem::take(&mut self.profile);
        let mut engine = SyncEngine::new(client, pool.clone(), profile);
        engine.set_cancel_flag(self.cancel_requested.clone());
        let result = engine.run(mode).await;
        self.profile = engine.into_profile();
        self.last_sync_result = Some(result.clone());

        if result.error.as_deref() != Some("Sync cancelled") {
            // Persist sync run summary
            let mut summary = SyncRunSummary::new();
            summary.completed_at = Some(crate::time::format_iso8601(crate::time::now_utc()));
            summary.total_items = result.pull_items;
            summary.errors_count = result.push_failed as usize
                + result.pull_errors
                + if result.error.is_some() { 1 } else { 0 };
            summary.is_complete = result.success;
            let repo = SqliteSyncRunRepo::new(pool);
            let _ = repo.insert_sync_run(&summary).await;
        }

        result
    }

    pub fn get_last_conflicts(&self) -> ConflictSummary {
        self.last_sync_result
            .as_ref()
            .map(|r| r.conflicts.clone())
            .unwrap_or_default()
    }

    // ── Online Mutations ─────────────────────────────────────────

    pub async fn approve_temp_item(
        &mut self,
        temp_id: i32,
        request: &crate::domain::temporary_items::ApproveAsItemRequest,
    ) -> CoreResult<crate::domain::temporary_items::TemporaryItemDto> {
        self.client()?
            .temporary_items_approve(temp_id, request)
            .await
    }

    pub async fn merge_temp_item(
        &mut self,
        temp_id: i32,
        request: &crate::domain::temporary_items::MergeToItemRequest,
    ) -> CoreResult<crate::domain::temporary_items::TemporaryItemDto> {
        self.client()?.temporary_items_merge(temp_id, request).await
    }

    pub async fn delete_temp_item(&mut self, temp_id: i32) -> CoreResult<()> {
        self.client()?.temporary_items_delete(temp_id).await
    }

    pub async fn generate_document(
        &mut self,
        request: &DocumentGenerateRequest,
    ) -> CoreResult<DocumentDto> {
        self.client()?.documents_generate(request).await
    }

    pub async fn render_document(&mut self, doc_id: uuid::Uuid) -> CoreResult<Vec<u8>> {
        self.client()?.documents_render(doc_id).await
    }

    pub async fn accept_operation_lines(
        &mut self,
        operation_id: &str,
        request: &AcceptLinesRequest,
    ) -> CoreResult<crate::domain::operation::OperationResponse> {
        self.client()?
            .operations_accept_lines(operation_id, request)
            .await
    }

    pub async fn resolve_lost_asset(
        &mut self,
        operation_line_id: &str,
        request: &LostAssetResolveRequest,
    ) -> CoreResult<serde_json::Value> {
        self.client()?
            .lost_assets_resolve(operation_line_id, request)
            .await
    }

    // ── Issue Objects ──────────────────────────────────────────────────

    pub async fn list_issue_objects(
        &mut self,
        page: u32,
        page_size: u32,
        search: Option<&str>,
    ) -> CoreResult<IssueObjectListResponse> {
        self.client()?
            .issue_objects_list(page, page_size, search)
            .await
    }

    pub async fn get_issue_object(&mut self, id: i32) -> CoreResult<IssueObjectDto> {
        self.client()?.issue_objects_get(id).await
    }

    pub async fn create_issue_object(
        &mut self,
        body: &IssueObjectCreate,
    ) -> CoreResult<IssueObjectDto> {
        self.client()?.issue_objects_create(body).await
    }

    pub async fn update_issue_object(
        &mut self,
        id: i32,
        body: &IssueObjectUpdate,
    ) -> CoreResult<IssueObjectDto> {
        self.client()?.issue_objects_update(id, body).await
    }

    pub async fn delete_issue_object(&mut self, id: i32) -> CoreResult<()> {
        self.client()?.issue_objects_delete(id).await
    }

    pub async fn merge_issue_objects(
        &mut self,
        body: &IssueObjectMerge,
    ) -> CoreResult<IssueObjectDto> {
        self.client()?.issue_objects_merge(body).await
    }

    pub async fn list_issue_object_assets(
        &mut self,
        id: i32,
        page: u32,
        page_size: u32,
    ) -> CoreResult<PaginatedResponse<serde_json::Value>> {
        self.client()?
            .issue_objects_list_assets(id, page, page_size)
            .await
    }

    pub async fn get_issue_object_tree(&mut self) -> CoreResult<Vec<IssueObjectTreeDto>> {
        self.client()?.issue_objects_get_tree().await
    }

    pub async fn create_issue_object_category(
        &mut self,
        body: &IssueObjectCategoryCreate,
    ) -> CoreResult<IssueObjectCategoryDto> {
        self.client()?.issue_object_categories_create(body).await
    }

    pub async fn list_issue_object_categories(
        &mut self,
        page: u32,
        page_size: u32,
    ) -> CoreResult<PaginatedResponse<IssueObjectCategoryDto>> {
        self.client()?
            .issue_object_categories_list(page, page_size)
            .await
    }

    pub async fn get_issue_object_category(
        &mut self,
        id: i32,
    ) -> CoreResult<IssueObjectCategoryDto> {
        self.client()?.issue_object_categories_get(id).await
    }

    pub async fn update_issue_object_category(
        &mut self,
        id: i32,
        body: &IssueObjectCategoryUpdate,
    ) -> CoreResult<IssueObjectCategoryDto> {
        self.client()?
            .issue_object_categories_update(id, body)
            .await
    }

    pub async fn delete_issue_object_category(&mut self, id: i32) -> CoreResult<()> {
        self.client()?.issue_object_categories_delete(id).await
    }

    // ── DELETE operation ───────────────────────────────────────────────

    pub async fn delete_operation(&mut self, id: &str) -> CoreResult<()> {
        self.client()?.operations_delete(id).await
    }
}

/// Snapshot of profile status for UI consumption.
#[derive(Debug, Clone)]
pub struct ProfileStatus {
    pub user_id: uuid::Uuid,
    pub user_name: String,
    pub user_email: String,
    pub role: String,
    pub is_root: bool,
    pub device_id: i32,
    pub device_registered: bool,
    pub active_site_id: Option<i32>,
    pub protocol_version: String,
    pub refreshed_at: String,
}

/// Auth context DTO for facade consumers.
#[derive(Debug, Clone)]
pub struct AuthContextDto {
    pub user_id: uuid::Uuid,
    pub user_name: String,
    pub user_email: String,
    pub role: String,
    pub is_root: bool,
    pub available_sites: Vec<crate::domain::auth::AuthSiteInfo>,
    pub device_id: i32,
    pub device_registered: bool,
}

/// Sync status info for UI.
#[derive(Debug, Clone)]
pub struct SyncStatusInfo {
    pub is_authenticated: bool,
    pub active_site_id: Option<i32>,
    pub outbox_pending: i64,
    pub cursor_count: i32,
}

/// Build a category tree from a flat list of CategoryDto.
fn build_category_tree(cats: &[CategoryDto]) -> Vec<CategoryTreeNode> {
    let now = crate::time::Timestamp::now_utc().to_string();
    let roots: Vec<&CategoryDto> = cats.iter().filter(|c| c.parent_id.is_none()).collect();
    roots.iter().map(|r| build_node(r, cats, &now)).collect()
}

fn build_node(cat: &CategoryDto, all: &[CategoryDto], now: &str) -> CategoryTreeNode {
    let children: Vec<&CategoryDto> = all.iter().filter(|c| c.parent_id == Some(cat.id)).collect();
    CategoryTreeNode {
        id: cat.id,
        name: cat.name.clone(),
        code: None,
        parent_id: cat.parent_id,
        is_active: cat.is_active,
        created_at: now.to_string(),
        updated_at: cat.updated_at.clone(),
        sort_order: None,
        path: Vec::new(),
        children: children.iter().map(|c| build_node(c, all, now)).collect(),
    }
}
