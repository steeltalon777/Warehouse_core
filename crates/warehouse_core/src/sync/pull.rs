use serde::{Deserialize, Serialize};

use crate::auth::ProfileService;
use crate::domain::pagination::PaginatedResponse;
use crate::error::{CoreError, CoreResult};
use crate::storage::cursor_store::CursorStore;
use crate::storage::cursor_store::keys;
use crate::storage::repos::{ErrorLogRepo, SqliteErrorLogRepo};
use crate::storage::snapshot_writer::SnapshotWriter;
use crate::syncserver::SyncServerClient;

/// Result of one pull family.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FamilyResult {
    pub name: String,
    pub success: bool,
    pub items_count: usize,
    pub error: Option<String>,
}

/// Summary of a sync/pull run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRunSummary {
    pub run_id: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub families: Vec<FamilyResult>,
    pub total_items: usize,
    pub errors_count: usize,
    pub is_complete: bool,
}

impl SyncRunSummary {
    pub fn new() -> Self {
        Self {
            run_id: uuid::Uuid::new_v4().to_string(),
            started_at: crate::time::Timestamp::now_utc().to_string(),
            completed_at: None,
            families: Vec::new(),
            total_items: 0,
            errors_count: 0,
            is_complete: false,
        }
    }

    pub fn finish(&mut self) {
        self.completed_at = Some(crate::time::Timestamp::now_utc().to_string());
        self.is_complete = true;
    }

    pub fn errors(&self) -> Vec<&str> {
        self.families
            .iter()
            .filter_map(|f| f.error.as_deref())
            .collect()
    }
}

impl Default for SyncRunSummary {
    fn default() -> Self {
        Self::new()
    }
}

/// PullSyncService: incremental refresh of all data families.
pub struct PullSyncService {
    client: SyncServerClient,
    profile: ProfileService,
    cursor_store: CursorStore,
    writer: SnapshotWriter,
    error_log: SqliteErrorLogRepo,
}

impl PullSyncService {
    pub fn new(
        client: SyncServerClient,
        profile: ProfileService,
        cursor_store: CursorStore,
        writer: SnapshotWriter,
        error_log: SqliteErrorLogRepo,
    ) -> Self {
        Self {
            client,
            profile,
            cursor_store,
            writer,
            error_log,
        }
    }

    /// Consume the service and return the ProfileService.
    pub fn into_profile(self) -> ProfileService {
        self.profile
    }

    // ── Main entry point ──────────────────────────────────

    /// Pull all data families. Returns a summary of successes and failures.
    /// Failed families do not affect already committed ones.
    pub async fn pull_all(&mut self) -> SyncRunSummary {
        let mut summary = SyncRunSummary::new();

        // 1. Health/readiness/protocol
        self.log_result(&mut summary, "health", self.pull_health().await)
            .await;

        // 2. Auth context
        self.log_result(&mut summary, "auth", self.pull_auth().await)
            .await;

        // 3. Sites
        self.log_result(&mut summary, "sites", self.pull_sites().await)
            .await;

        // 4. Catalog (categories first, then units, then items — FK dependency order)
        self.log_result(
            &mut summary,
            "catalog_categories",
            self.pull_catalog_categories().await,
        )
        .await;
        self.log_result(
            &mut summary,
            "catalog_units",
            self.pull_catalog_units().await,
        )
        .await;
        self.log_result(
            &mut summary,
            "catalog_items",
            self.pull_catalog_items().await,
        )
        .await;

        // 5. Recipients
        self.log_result(&mut summary, "recipients", self.pull_recipients().await)
            .await;

        // 6. Balances for active site
        if let Ok(site_ids) = self.profile.available_site_ids() {
            for sid in site_ids {
                self.log_result(&mut summary, "balances", self.pull_balances(sid).await)
                    .await;
            }
        }

        // 7. Assets
        self.log_result(
            &mut summary,
            "pending_acceptance",
            self.pull_pending_acceptance().await,
        )
        .await;
        self.log_result(&mut summary, "lost_assets", self.pull_lost_assets().await)
            .await;
        self.log_result(
            &mut summary,
            "issued_assets",
            self.pull_issued_assets().await,
        )
        .await;

        // 8. Temporary items
        self.log_result(
            &mut summary,
            "temporary_items",
            self.pull_temporary_items().await,
        )
        .await;

        // 9. Operation history (for active site)
        if let Some(site_id) = self.get_active_site() {
            self.log_result(
                &mut summary,
                "operations",
                self.pull_operations(site_id).await,
            )
            .await;
        }

        // 10. Documents (for active site)
        if let Some(site_id) = self.get_active_site() {
            self.log_result(
                &mut summary,
                "documents",
                self.pull_documents(site_id).await,
            )
            .await;
        }

        // 11. Stock summary report cache
        self.log_result(
            &mut summary,
            "stock_summary",
            self.pull_stock_summary().await,
        )
        .await;

        // 12. Device pull event stream (server_seq)
        self.log_result(
            &mut summary,
            "device_events",
            self.pull_device_events().await,
        )
        .await;

        summary.finish();
        summary
    }

    /// Pull a single family by name (for targeted refresh).
    pub async fn pull_family(&mut self, family: &str) -> CoreResult<FamilyResult> {
        match family {
            "health" => self.pull_health().await,
            "auth" => self.pull_auth().await,
            "sites" => self.pull_sites().await,
            "catalog_items" => self.pull_catalog_items().await,
            "catalog_categories" => self.pull_catalog_categories().await,
            "catalog_units" => self.pull_catalog_units().await,
            "recipients" => self.pull_recipients().await,
            "balances" => {
                if let Ok(site_ids) = self.profile.available_site_ids() {
                    for sid in &site_ids {
                        self.pull_balances(*sid).await?;
                    }
                    Ok(FamilyResult {
                        name: "balances".into(),
                        success: true,
                        items_count: 0,
                        error: None,
                    })
                } else {
                    Ok(FamilyResult {
                        name: "balances".into(),
                        success: false,
                        items_count: 0,
                        error: Some("No profile".into()),
                    })
                }
            }
            "pending_acceptance" => self.pull_pending_acceptance().await,
            "lost_assets" => self.pull_lost_assets().await,
            "issued_assets" => self.pull_issued_assets().await,
            "temporary_items" => self.pull_temporary_items().await,
            "operations" => {
                if let Some(site_id) = self.get_active_site() {
                    self.pull_operations(site_id).await
                } else {
                    Ok(FamilyResult {
                        name: "operations".into(),
                        success: false,
                        items_count: 0,
                        error: Some("No active site".into()),
                    })
                }
            }
            "documents" => {
                if let Some(site_id) = self.get_active_site() {
                    self.pull_documents(site_id).await
                } else {
                    Ok(FamilyResult {
                        name: "documents".into(),
                        success: false,
                        items_count: 0,
                        error: Some("No active site".into()),
                    })
                }
            }
            "stock_summary" => self.pull_stock_summary().await,
            "device_events" => self.pull_device_events().await,
            _ => Err(CoreError::Validation(format!(
                "Unknown pull family: {family}"
            ))),
        }
    }

    // ── Per-family pull methods ───────────────────────────

    async fn pull_health(&self) -> CoreResult<FamilyResult> {
        let info = self.client.server_info().await?;
        Ok(FamilyResult {
            name: "health".into(),
            success: info.status == "ok" || info.status == "healthy",
            items_count: 1,
            error: None,
        })
    }

    async fn pull_auth(&self) -> CoreResult<FamilyResult> {
        let ctx = self.client.auth_context().await?;
        // Auth context is already handled by ProfileService in Level 2,
        // but we check connectivity here.
        let _ = ctx;
        Ok(FamilyResult {
            name: "auth".into(),
            success: true,
            items_count: 0,
            error: None,
        })
    }

    async fn pull_sites(&self) -> CoreResult<FamilyResult> {
        let sites = self.client.catalog_sites().await?;
        let count = sites.len();
        self.writer.write_sites(&sites).await?;
        self.cursor_store
            .set_updated_after(keys::SITES, &crate::time::Timestamp::now_utc().to_string())
            .await?;
        Ok(FamilyResult {
            name: "sites".into(),
            success: true,
            items_count: count,
            error: None,
        })
    }

    async fn pull_catalog_items(&self) -> CoreResult<FamilyResult> {
        let cursor = self
            .cursor_store
            .get_updated_after(keys::CATALOG_ITEMS)
            .await?;
        let resp = self
            .client
            .catalog_items(cursor.as_deref(), Some(200))
            .await?;
        let count = resp.items.len();
        self.writer.write_items(&resp.items).await?;
        if let Some(next) = &resp.next_updated_after {
            self.cursor_store
                .set_updated_after(keys::CATALOG_ITEMS, next)
                .await?;
        }
        Ok(FamilyResult {
            name: "catalog_items".into(),
            success: true,
            items_count: count,
            error: None,
        })
    }

    async fn pull_catalog_categories(&self) -> CoreResult<FamilyResult> {
        let cursor = self
            .cursor_store
            .get_updated_after(keys::CATALOG_CATEGORIES)
            .await?;
        let resp = self
            .client
            .catalog_categories(cursor.as_deref(), Some(200))
            .await?;
        let count = resp.items.len();
        self.writer.write_categories(&resp.items).await?;
        if let Some(next) = &resp.next_updated_after {
            self.cursor_store
                .set_updated_after(keys::CATALOG_CATEGORIES, next)
                .await?;
        }
        Ok(FamilyResult {
            name: "catalog_categories".into(),
            success: true,
            items_count: count,
            error: None,
        })
    }

    async fn pull_catalog_units(&self) -> CoreResult<FamilyResult> {
        let cursor = self
            .cursor_store
            .get_updated_after(keys::CATALOG_UNITS)
            .await?;
        let resp = self
            .client
            .catalog_units(cursor.as_deref(), Some(200))
            .await?;
        let count = resp.items.len();
        self.writer.write_units(&resp.items).await?;
        if let Some(next) = &resp.next_updated_after {
            self.cursor_store
                .set_updated_after(keys::CATALOG_UNITS, next)
                .await?;
        }
        Ok(FamilyResult {
            name: "catalog_units".into(),
            success: true,
            items_count: count,
            error: None,
        })
    }

    async fn pull_recipients(&self) -> CoreResult<FamilyResult> {
        // Recipients endpoint may not exist on all SyncServer versions;
        // treat 404 as empty (the feature is Django-managed).
        match self._pull_recipients_inner().await {
            Ok(result) => Ok(result),
            Err(crate::error::CoreError::NotFound(_)) => Ok(FamilyResult {
                name: "recipients".into(),
                success: true,
                items_count: 0,
                error: None,
            }),
            Err(e) => Err(e),
        }
    }

    async fn _pull_recipients_inner(&self) -> CoreResult<FamilyResult> {
        // Page through all recipients
        let mut all: Vec<crate::domain::recipient::RecipientDto> = Vec::new();
        let page_size = 200u32;
        let mut page = 1u32;
        loop {
            let resp: PaginatedResponse<crate::domain::recipient::RecipientDto> =
                self.client.recipients_list(page, page_size, None).await?;
            let count = resp.items.len();
            all.extend(resp.items);
            if count < page_size as usize || page * page_size >= resp.total_count as u32 {
                break;
            }
            page += 1;
        }
        self.writer.write_recipients(&all).await?;
        self.cursor_store
            .set_updated_after(
                keys::RECIPIENTS,
                &crate::time::Timestamp::now_utc().to_string(),
            )
            .await?;
        Ok(FamilyResult {
            name: "recipients".into(),
            success: true,
            items_count: all.len(),
            error: None,
        })
    }

    async fn pull_balances(&self, site_id: i32) -> CoreResult<FamilyResult> {
        let mut all: Vec<crate::domain::balance::BalanceRow> = Vec::new();
        let page_size = 200u32;
        let mut page = 1u32;
        loop {
            let resp: PaginatedResponse<crate::domain::balance::BalanceRow> = self
                .client
                .balances_by_site(site_id, page, page_size)
                .await?;
            let count = resp.items.len();
            all.extend(resp.items);
            if count < page_size as usize || page * page_size >= resp.total_count as u32 {
                break;
            }
            page += 1;
        }
        self.writer.write_balances(site_id, &all).await?;
        self.cursor_store
            .set_updated_after(
                &keys::balances(site_id),
                &crate::time::Timestamp::now_utc().to_string(),
            )
            .await?;
        Ok(FamilyResult {
            name: "balances".into(),
            success: true,
            items_count: all.len(),
            error: None,
        })
    }

    async fn pull_pending_acceptance(&self) -> CoreResult<FamilyResult> {
        let mut all: Vec<crate::domain::assets::PendingAcceptanceRow> = Vec::new();
        let page_size = 200u32;
        let mut page = 1u32;
        loop {
            let resp: PaginatedResponse<crate::domain::assets::PendingAcceptanceRow> =
                self.client.pending_acceptance_list(page, page_size).await?;
            let count = resp.items.len();
            all.extend(resp.items);
            if count < page_size as usize || page * page_size >= resp.total_count as u32 {
                break;
            }
            page += 1;
        }
        self.writer.write_pending_acceptance(&all).await?;
        self.cursor_store
            .set_updated_after(
                keys::PENDING_ACCEPTANCE,
                &crate::time::Timestamp::now_utc().to_string(),
            )
            .await?;
        Ok(FamilyResult {
            name: "pending_acceptance".into(),
            success: true,
            items_count: all.len(),
            error: None,
        })
    }

    async fn pull_lost_assets(&self) -> CoreResult<FamilyResult> {
        let mut all: Vec<crate::domain::assets::LostAssetRow> = Vec::new();
        let page_size = 200u32;
        let mut page = 1u32;
        loop {
            let resp: PaginatedResponse<crate::domain::assets::LostAssetRow> =
                self.client.lost_assets_list(page, page_size, None).await?;
            let count = resp.items.len();
            all.extend(resp.items);
            if count < page_size as usize || page * page_size >= resp.total_count as u32 {
                break;
            }
            page += 1;
        }
        self.writer.write_lost_assets(&all).await?;
        self.cursor_store
            .set_updated_after(
                keys::LOST_ASSETS,
                &crate::time::Timestamp::now_utc().to_string(),
            )
            .await?;
        Ok(FamilyResult {
            name: "lost_assets".into(),
            success: true,
            items_count: all.len(),
            error: None,
        })
    }

    async fn pull_issued_assets(&self) -> CoreResult<FamilyResult> {
        let mut all: Vec<crate::domain::assets::IssuedAssetRow> = Vec::new();
        let page_size = 200u32;
        let mut page = 1u32;
        loop {
            let resp: PaginatedResponse<crate::domain::assets::IssuedAssetRow> = self
                .client
                .issued_assets_list(page, page_size, None)
                .await?;
            let count = resp.items.len();
            all.extend(resp.items);
            if count < page_size as usize || page * page_size >= resp.total_count as u32 {
                break;
            }
            page += 1;
        }
        self.writer.write_issued_assets(&all).await?;
        self.cursor_store
            .set_updated_after(
                keys::ISSUED_ASSETS,
                &crate::time::Timestamp::now_utc().to_string(),
            )
            .await?;
        Ok(FamilyResult {
            name: "issued_assets".into(),
            success: true,
            items_count: all.len(),
            error: None,
        })
    }

    async fn pull_temporary_items(&self) -> CoreResult<FamilyResult> {
        let mut all: Vec<crate::domain::temporary_items::TemporaryItemDto> = Vec::new();
        let page_size = 100u32;
        let mut page = 1u32;
        loop {
            let resp: PaginatedResponse<crate::domain::temporary_items::TemporaryItemDto> =
                self.client.temporary_items_list(page, page_size).await?;
            let count = resp.items.len();
            all.extend(resp.items);
            if count < page_size as usize || page * page_size >= resp.total_count as u32 {
                break;
            }
            page += 1;
        }
        self.writer.write_temporary_items(&all).await?;
        self.cursor_store
            .set_updated_after(
                keys::TEMPORARY_ITEMS,
                &crate::time::Timestamp::now_utc().to_string(),
            )
            .await?;
        Ok(FamilyResult {
            name: "temporary_items".into(),
            success: true,
            items_count: all.len(),
            error: None,
        })
    }

    async fn pull_operations(&self, site_id: i32) -> CoreResult<FamilyResult> {
        let mut all: Vec<crate::domain::operation::OperationListItem> = Vec::new();
        let page_size = 100u32;
        let mut page = 1u32;
        let filters: Vec<(&str, String)> = vec![("site_id", site_id.to_string())];
        loop {
            let filter_map: std::collections::HashMap<&str, &str> =
                filters.iter().map(|(k, v)| (*k, v.as_str())).collect();
            let resp: PaginatedResponse<crate::domain::operation::OperationListItem> = self
                .client
                .operations_list(page, page_size, Some(filter_map))
                .await?;
            let count = resp.items.len();
            all.extend(resp.items);
            if count < page_size as usize
                || page * page_size >= resp.total_count as u32
                || page >= 5
            {
                break;
            }
            page += 1;
        }
        self.writer.write_operations(&all).await?;
        self.cursor_store
            .set_updated_after(
                keys::OPERATIONS,
                &crate::time::Timestamp::now_utc().to_string(),
            )
            .await?;
        Ok(FamilyResult {
            name: "operations".into(),
            success: true,
            items_count: all.len(),
            error: None,
        })
    }

    async fn pull_documents(&self, site_id: i32) -> CoreResult<FamilyResult> {
        let mut all: Vec<crate::domain::documents::DocumentDto> = Vec::new();
        let limit = 100u64;
        let mut offset = 0u64;
        loop {
            let resp: PaginatedResponse<crate::domain::documents::DocumentDto> =
                self.client.documents_list(offset, limit).await?;
            let count = resp.items.len();
            all.extend(resp.items);
            if count < limit as usize {
                break;
            }
            offset += limit;
        }
        // Filter to site docs after fetching
        let site_docs: Vec<_> = all.into_iter().filter(|d| d.site_id == site_id).collect();
        self.writer.write_documents(&site_docs).await?;
        self.cursor_store
            .set_updated_after(
                keys::DOCUMENTS,
                &crate::time::Timestamp::now_utc().to_string(),
            )
            .await?;
        Ok(FamilyResult {
            name: "documents".into(),
            success: true,
            items_count: site_docs.len(),
            error: None,
        })
    }

    async fn pull_stock_summary(&self) -> CoreResult<FamilyResult> {
        let resp = self.client.reports_stock_summary(1, 200).await?;
        let params_hash = format!("stock_summary_{}", crate::time::Timestamp::now_utc());
        self.writer
            .write_stock_summary(&params_hash, &resp.items)
            .await?;
        self.cursor_store
            .set_updated_after(
                keys::STOCK_SUMMARY,
                &crate::time::Timestamp::now_utc().to_string(),
            )
            .await?;
        Ok(FamilyResult {
            name: "stock_summary".into(),
            success: true,
            items_count: resp.items.len(),
            error: None,
        })
    }

    async fn pull_device_events(&self) -> CoreResult<FamilyResult> {
        // Device pull requires registered device token; if device is not
        // registered (e.g. fake token), skip gracefully.
        if !self.profile.is_authenticated()
            || !self
                .profile
                .current()
                .ok()
                .is_some_and(|p| p.device_registered)
        {
            return Ok(FamilyResult {
                name: "device_events".into(),
                success: true,
                items_count: 0,
                error: None,
            });
        }
        let seq = self.cursor_store.get_server_seq().await?.unwrap_or(0);
        let req = crate::domain::sync_types::PullRequest {
            site_id: self.get_active_site().unwrap_or(0),
            device_id: 0,
            since_seq: seq,
            limit: Some(200),
        };
        match self.client.pull(&req).await {
            Ok(resp) => {
                self.cursor_store
                    .set_server_seq(resp.server_seq_upto)
                    .await?;
                Ok(FamilyResult {
                    name: "device_events".into(),
                    success: true,
                    items_count: resp.events.len(),
                    error: None,
                })
            }
            Err(crate::error::CoreError::Auth(_)) => {
                // Device not authenticated; skip gracefully
                Ok(FamilyResult {
                    name: "device_events".into(),
                    success: true,
                    items_count: 0,
                    error: None,
                })
            }
            Err(e) => Err(e),
        }
    }

    // ── Helpers ───────────────────────────────────────────

    fn get_active_site(&self) -> Option<i32> {
        self.profile.current().ok()?.active_site_id
    }

    async fn log_result(
        &self,
        summary: &mut SyncRunSummary,
        family_name: &'static str,
        result: CoreResult<FamilyResult>,
    ) {
        match result {
            Ok(family) => {
                let count = family.items_count;
                summary.families.push(family);
                summary.total_items += count;
            }
            Err(e) => {
                let err_str = e.to_string();
                summary.families.push(FamilyResult {
                    name: family_name.to_string(),
                    success: false,
                    items_count: 0,
                    error: Some(err_str.clone()),
                });
                summary.errors_count += 1;
                let _ = self.error_log.log("ERROR", "sync", &err_str, None).await;
            }
        }
    }
}
