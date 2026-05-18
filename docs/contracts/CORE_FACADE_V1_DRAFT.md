# CoreFacade V1 Draft

> Target: Stable UI-facing API for Kotlin/C#/CLI.
> Version: v1-draft
> Status: Draft for Level 0 review

## Design Principles

1. **No internal types leak** — no `sqlx`, `reqwest`, or `tokio` types in facade signatures.
2. **All returns are `CoreResult<T>`** where `T` is a domain DTO or `()`.
3. **Async operations** return handle/observable for progress (FFI-compatible).
4. **Token injection** is a one-time bind, not per-call parameter.
5. **Active site** is profile state, not per-call parameter.

## Lifecycle

```rust
pub struct CoreFacade {
    // Internal: repo_bag, http_client, sync_engine, draft_service
}

impl CoreFacade {
    /// Open existing profile or create new.
    pub async fn open(config: CoreConfig) -> CoreResult<Self>;

    /// Open in read-only diagnostics mode.
    pub async fn open_readonly(profile_path: &str) -> CoreResult<Self>;

    /// Close profile, flush pending writes.
    pub async fn close(self) -> CoreResult<()>;

    /// Profile health (local DB, no network).
    pub fn health_local(&self) -> HealthReport;

    /// Diagnostic dump.
    pub fn diagnostics(&self) -> DiagnosticsReport;
}
```

## Auth & Site

```rust
impl CoreFacade {
    /// Bind user token. Core stores identity hash, not raw token.
    pub async fn bind_user_token(&self, token: &str) -> CoreResult<AuthContext>;

    /// Refresh identity from server.
    pub async fn refresh_identity(&self) -> CoreResult<AuthContext>;

    /// Get cached auth context.
    pub fn get_auth_context(&self) -> Option<AuthContext>;

    /// List available sites for current user.
    pub fn list_available_sites(&self) -> Vec<AuthSiteInfo>;

    /// Set active site. Fails if user lacks access.
    pub async fn set_active_site(&self, site_id: i32) -> CoreResult<()>;

    /// Get active site.
    pub fn get_active_site(&self) -> Option<SiteDto>;
}
```

## Connectivity

```rust
impl CoreFacade {
    /// Simple health check.
    pub async fn health_remote(&self) -> CoreResult<HealthStatus>;

    /// Full readiness.
    pub async fn ready_remote(&self) -> CoreResult<HealthCheckResponse>;
}
```

## Sync

```rust
impl CoreFacade {
    /// Full bootstrap: auth + catalogs + balances + recipients.
    pub async fn bootstrap(&self) -> CoreResult<SyncSummary>;

    /// Incremental pull of all families.
    pub async fn pull_once(&self) -> CoreResult<SyncSummary>;

    /// Current sync status.
    pub fn get_sync_status(&self) -> SyncStatus;

    /// Recent sync runs.
    pub fn list_sync_runs(&self) -> Vec<SyncRunSummary>;
}
```

## Catalog (Read)

```rust
impl CoreFacade {
    /// Search items by name/SKU.
    pub fn search_items(&self, query: &str, limit: u32) -> CoreResult<Vec<ItemDto>>;

    /// Get single item.
    pub fn get_item(&self, id: i32) -> CoreResult<Option<ItemDto>>;

    /// All units.
    pub fn list_units(&self) -> CoreResult<Vec<UnitDto>>;

    /// All categories.
    pub fn list_categories(&self) -> CoreResult<Vec<CategoryDto>>;

    /// Full category tree.
    pub fn get_category_tree(&self) -> CoreResult<Vec<CategoryTreeNode>>;

    /// Category parent chain (breadcrumb).
    pub fn get_parent_chain(&self, category_id: i32) -> CoreResult<Vec<CategoryDto>>;
}
```

## Balances & Assets (Read)

```rust
impl CoreFacade {
    /// All balances for active site.
    pub fn list_balances(&self) -> CoreResult<Vec<BalanceRow>>;

    /// Balance for specific item.
    pub fn get_balance_by_subject(&self, item_id: i32) -> CoreResult<Vec<BalanceRow>>;

    /// Stock summary.
    pub fn get_balance_summary(&self) -> CoreResult<Vec<BalanceSummaryRow>>;

    /// Pending acceptance.
    pub fn list_pending_acceptance(&self) -> CoreResult<Vec<PendingAcceptanceRow>>;

    /// Lost assets.
    pub fn list_lost_assets(&self) -> CoreResult<Vec<LostAssetRow>>;

    /// Issued assets.
    pub fn list_issued_assets(&self) -> CoreResult<Vec<IssuedAssetRow>>;
}
```

## Operations (Read + Draft)

```rust
impl CoreFacade {
    // Read
    pub fn list_operations(&self, filter: OperationFilter) -> CoreResult<Vec<OperationListItem>>;
    pub fn get_operation(&self, id: i64) -> CoreResult<Option<OperationResponse>>;

    // Drafts
    pub fn create_draft(&self, draft: OperationDraft) -> CoreResult<()>;
    pub fn get_draft(&self, draft_id: Uuid) -> CoreResult<Option<OperationDraft>>;
    pub fn update_draft(&self, draft: OperationDraft) -> CoreResult<()>;
    pub fn delete_draft(&self, draft_id: Uuid) -> CoreResult<()>;
    pub fn list_drafts(&self) -> CoreResult<Vec<OperationDraft>>;
    pub fn validate_draft(&self, draft_id: Uuid) -> CoreResult<Vec<DraftValidationError>>;
}
```

## Recipients (Read)

```rust
impl CoreFacade {
    pub fn search_recipients(&self, query: &str) -> CoreResult<Vec<RecipientDto>>;
    pub fn get_recipient(&self, id: i32) -> CoreResult<Option<RecipientDto>>;
}
```

## Temporary Items (Read + Moderation)

```rust
impl CoreFacade {
    pub fn list_temporary_items(&self) -> CoreResult<Vec<TemporaryItemDto>>;
    pub fn get_temporary_item(&self, id: i32) -> CoreResult<Option<TemporaryItemDto>>;
}
```

## Documents & Reports (Read)

```rust
impl CoreFacade {
    pub fn list_documents(&self, filter: DocumentFilter) -> CoreResult<Vec<DocumentDto>>;
    pub fn get_document(&self, id: Uuid) -> CoreResult<Option<DocumentDto>>;
    pub fn list_operation_documents(&self, operation_id: i64) -> CoreResult<Vec<DocumentDto>>;
    pub fn run_stock_summary(&self) -> CoreResult<Vec<StockSummaryRow>>;
    pub fn run_item_movement(&self, filter: ItemMovementFilter) -> CoreResult<Vec<ItemMovementRow>>;
}
```

## Outbox & Submit

```rust
impl CoreFacade {
    /// Queue a draft for submission.
    pub async fn queue_draft(&self, draft_id: Uuid) -> CoreResult<OutboxEvent>;

    /// Submit a draft immediately (online).
    pub async fn submit_now(&self, draft_id: Uuid) -> CoreResult<OperationResponse>;

    /// List outbox events.
    pub fn list_outbox_events(&self, filter: OutboxFilter) -> CoreResult<Vec<OutboxEvent>>;

    /// Retry a failed outbox event.
    pub async fn retry_outbox_event(&self, event_uuid: Uuid) -> CoreResult<()>;

    /// Cancel a pending outbox event.
    pub async fn cancel_outbox_event(&self, event_uuid: Uuid) -> CoreResult<()>;
}
```

## Sync Engine

```rust
impl CoreFacade {
    /// Full sync: push pending → pull updates.
    pub async fn sync_once(&self) -> CoreResult<SyncSummary>;

    /// Get unresolved conflicts.
    pub fn list_conflicts(&self) -> CoreResult<Vec<ConflictEntry>>;

    /// Acknowledge/resolve conflict.
    pub async fn resolve_conflict(&self, conflict_id: Uuid) -> CoreResult<()>;
}
```

## Error DTO

```rust
pub struct CoreErrorDto {
    pub code: String,       // Machine-readable: "unauthenticated", "validation", "conflict"
    pub message: String,    // Human-readable
    pub details: Option<String>,
}
```

## Futures / Notes

- FFI: `CoreFacade` is wrapped in `Box<dyn CoreFacadeTrait>` + `Arc<Mutex<>>` for UniFFI.
- JSON dynamic fields (`qty`, `payload`) remain as `serde_json::Value` in Rust; FFI layer converts to string.
- Draft validation errors map to separate FFI struct with field-level error codes.
- Sync progress observable: TBD whether push or poll model for FFI.
