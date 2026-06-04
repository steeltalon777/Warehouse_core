# CoreFacade V1 — Full Method / DTO / Error Reference

> **Version:** 0.3.0  
> **Updated:** 2026-06-04  
> **Total public methods:** 92 (87 domain methods + 5 internal helpers)

---

## Table of Contents

- [Lifecycle](#lifecycle)
- [Auth / Site](#auth--site)
- [Connectivity](#connectivity)
- [Sync](#sync)
- [Catalog](#catalog)
- [Balances / Assets](#balances--assets)
- [Operations Read](#operations-read)
- [Recipients](#recipients)
- [Temporary Items](#temporary-items)
- [Documents / Reports](#documents--reports)
- [Drafts](#drafts)
- [Outbox](#outbox)
- [Sync Engine](#sync-engine)
- [Online Mutations](#online-mutations)
- [Issue Objects](#issue-objects)
- [DTOs](#dtos)
- [FFI Error Codes](#ffi-error-codes)
- [CoreConfig](#coreconfig)
- [TokenProvider](#tokenprovider)

---

## Lifecycle

| Method | Signature | Description |
|---|---|---|
| `open` | `pub async fn open(config: CoreConfig) -> CoreResult<Self>` | Open a writable profile from config. Creates/opens SQLite DB, initialises `CoreHandle` with null client and null token provider. |
| `open_readonly` | `pub async fn open_readonly(db_path: &str) -> CoreResult<Self>` | Open an existing SQLite DB in read-only mode. Uses a test-oriented config with no server URL. |
| `close` | `pub fn close(&mut self)` | Release the HTTP client (drop `Option<SyncServerClient>`). Profile and DB remain open. |
| `set_token_provider` | `pub fn set_token_provider(&mut self, provider: Box<dyn TokenProvider>)` | Wire a platform token provider. Used before any remote call. |
| `config` | `pub fn config(&self) -> &CoreConfig` | *(internal)* Return reference to current config. |
| `db` | `pub fn db(&self) -> &Database` | *(internal)* Return reference to SQLite `Database`. |
| `client` | `pub fn client(&mut self) -> CoreResult<&SyncServerClient>` | *(internal)* Lazy-init `SyncServerClient` with tokens from provider. Returns existing or creates new. |
| `profile` | `pub fn profile(&self) -> &ProfileService` | *(internal)* Return reference to `ProfileService`. |
| `profile_mut` | `pub fn profile_mut(&mut self) -> &mut ProfileService` | *(internal)* Return mutable reference to `ProfileService`. |
| `profile_status` | `pub fn profile_status(&self) -> CoreResult<ProfileStatus>` | Snapshot of current profile: user_id, name, email, role, is_root, device_id, device_registered, active_site_id, protocol_version, refreshed_at. |
| `health_local` | `pub fn health_local(&self) -> CoreResult<String>` | Probe local SQLite DB connectivity. Returns `"OK"` on success. |
| `diagnostics` | `pub async fn diagnostics(&self) -> CoreResult<DiagnosticsInfo>` | Full diagnostics: config JSON, db_path, profile exists, profile user/role, active_site, outbox_pending count, client version. |

## Auth / Site

| Method | Signature | Description |
|---|---|---|
| `load_profile` | `pub async fn load_profile(&mut self) -> CoreResult<()>` | Load persisted identity/profile from SQLite key-value storage. Must be called after `open` if a profile exists. |
| `refresh_identity` | `pub async fn refresh_identity(&mut self) -> CoreResult<()>` | Call `GET /auth/context` on SyncServer, persist the result to SQLite, update in-memory profile. |
| `get_auth_context` | `pub fn get_auth_context(&self) -> CoreResult<AuthContextDto>` | Return current auth context: user_id, name, email, role, is_root, available_sites, device_id, device_registered. |
| `list_available_sites` | `pub fn list_available_sites(&self) -> CoreResult<Vec<AuthSiteInfo>>` | List sites available to the current user from cached profile. |
| `set_active_site` | `pub async fn set_active_site(&mut self, site_id: i32) -> CoreResult<()>` | Validate site_id against available sites, persist to SQLite, update in-memory. |
| `get_active_site` | `pub fn get_active_site(&self) -> CoreResult<Option<i32>>` | Return currently active site ID from profile, or `None`. |
| `clear_active_site` | `pub async fn clear_active_site(&mut self) -> CoreResult<()>` | Clear active site from profile and SQLite. |
| `logout` | `pub async fn logout(&mut self) -> CoreResult<()>` | Clear profile from SQLite, drop in-memory profile, drop HTTP client. |

## Connectivity

| Method | Signature | Description |
|---|---|---|
| `health_remote` | `pub async fn health_remote(&mut self) -> CoreResult<bool>` | Call `GET /api/v1/health`. Returns `true` if status is `"ok"` or `"healthy"`. |
| `ready_remote` | `pub async fn ready_remote(&mut self) -> CoreResult<String>` | Call `GET /api/v1/ready`. Returns the status string from the server. |

## Sync

| Method | Signature | Description |
|---|---|---|
| `bootstrap_device` | `pub async fn bootstrap_device(&mut self) -> CoreResult<Profile>` | Call `POST /bootstrap/sync`. Build profile from bootstrap response (root_user, device_id, available_sites, protocol_version). Save to SQLite. |
| `bootstrap` | `pub async fn bootstrap(&mut self) -> CoreResult<BootstrapResult>` | Multi-family bootstrap: runs `BootstrapService` with cursor store and snapshot writer. Pulls auth context, sites, catalog, recipients, etc. Saves updated profile. |
| `pull_once` | `pub async fn pull_once(&mut self) -> SyncRunSummary` | Run `PullSyncService.pull_all()` — pulls 12 families (health, auth, sites, catalog, recipients, balances, assets, temp_items, operations, documents, reports, device_events). Returns per-family summary. |
| `get_sync_status` | `pub async fn get_sync_status(&self) -> CoreResult<SyncStatusInfo>` | Return sync status: is_authenticated, active_site_id, outbox_pending count, cursor_count. |
| `list_sync_runs` | `pub async fn list_sync_runs(&self) -> CoreResult<Vec<SyncRunSummary>>` | List recent sync runs from local SQLite (up to 20). |

## Catalog

| Method | Signature | Description |
|---|---|---|
| `search_items` | `pub async fn search_items(&self, query: &str) -> CoreResult<Vec<ItemDto>>` | Search catalog items by name/code/SKU in local SQLite. |
| `get_item` | `pub async fn get_item(&self, id: i32) -> CoreResult<Option<ItemDto>>` | Get catalog item by ID from local SQLite. |
| `list_units` | `pub async fn list_units(&self) -> CoreResult<Vec<UnitDto>>` | List all measurement units from local SQLite. |
| `list_categories` | `pub async fn list_categories(&self) -> CoreResult<Vec<CategoryDto>>` | List all categories from local SQLite (flat list). |
| `get_category_tree` | `pub async fn get_category_tree(&self) -> CoreResult<Vec<CategoryTreeNode>>` | Build recursive category tree from local SQLite data. |
| `get_parent_chain` | `pub async fn get_parent_chain(&self, category_id: i32) -> CoreResult<Vec<CategoryDto>>` | Build ancestor chain (root → ... → category) from local SQLite data. |

## Balances / Assets

| Method | Signature | Description |
|---|---|---|
| `list_balances` | `pub async fn list_balances(&self, site_id: i32) -> CoreResult<Vec<BalanceRow>>` | List balances for a site from local SQLite. |
| `get_balance_by_subject` | `pub async fn get_balance_by_subject(&self, item_id: i32) -> CoreResult<Vec<BalanceRow>>` | List balances for a specific item across sites from local SQLite. |
| `get_balance_summary` | `pub async fn get_balance_summary(&mut self) -> CoreResult<BalanceSummaryRow>` | Fetch balance summary online from SyncServer. |
| `list_pending_acceptance` | `pub async fn list_pending_acceptance(&self) -> CoreResult<Vec<PendingAcceptanceRow>>` | List pending acceptance rows from local SQLite. |
| `list_lost_assets` | `pub async fn list_lost_assets(&self) -> CoreResult<Vec<LostAssetRow>>` | List lost asset records from local SQLite. |
| `list_issued_assets` | `pub async fn list_issued_assets(&self) -> CoreResult<Vec<IssuedAssetRow>>` | List issued asset records from local SQLite. |

## Operations Read

| Method | Signature | Description |
|---|---|---|
| `list_operations` | `pub async fn list_operations(&mut self, site_id: i32, page: u32, page_size: u32) -> CoreResult<PaginatedResponse<OperationListItem>>` | List operations from SyncServer (online). Filtered by site_id. Paginated (max page_size 100). |
| `get_operation` | `pub async fn get_operation(&mut self, operation_id: &str) -> CoreResult<OperationResponse>` | Get operation detail from SyncServer (online) by ID. |
| `delete_operation` | `pub async fn delete_operation(&mut self, id: &str) -> CoreResult<()>` | Delete an operation on SyncServer (online). |

## Recipients

| Method | Signature | Description |
|---|---|---|
| `search_recipients` | `pub async fn search_recipients(&self, query: &str) -> CoreResult<Vec<RecipientDto>>` | Search recipients by name/code in local SQLite. |
| `get_recipient` | `pub async fn get_recipient(&self, id: i32) -> CoreResult<Option<RecipientDto>>` | Get recipient by ID from local SQLite. |

## Temporary Items

| Method | Signature | Description |
|---|---|---|
| `list_temporary_items` | `pub async fn list_temporary_items(&self) -> CoreResult<Vec<TemporaryItemDto>>` | List active temporary items from local SQLite. |
| `get_temporary_item` | `pub async fn get_temporary_item(&self, id: i32) -> CoreResult<Option<TemporaryItemDto>>` | Get temporary item by ID from local SQLite. |
| `list_temporary_item_operations` | `pub async fn list_temporary_item_operations(&mut self, temp_id: i32) -> CoreResult<PaginatedResponse<OperationListItem>>` | List operations referencing a temporary item (online). |

## Documents / Reports

| Method | Signature | Description |
|---|---|---|
| `list_documents` | `pub async fn list_documents(&self, site_id: i32) -> CoreResult<Vec<DocumentDto>>` | List document metadata for a site from local SQLite. |
| `get_document` | `pub async fn get_document(&self, id: &str) -> CoreResult<Option<DocumentDto>>` | Get document metadata by ID from local SQLite. |
| `list_operation_documents` | `pub async fn list_operation_documents(&mut self, operation_id: Uuid) -> CoreResult<Vec<DocumentDto>>` | List documents linked to an operation (online). |
| `generate_document` | `pub async fn generate_document(&mut self, request: &DocumentGenerateRequest) -> CoreResult<DocumentDto>` | Request document generation on SyncServer (online). |
| `render_document` | `pub async fn render_document(&mut self, doc_id: Uuid) -> CoreResult<Vec<u8>>` | Fetch rendered document bytes from SyncServer (online, raw PDF/print data). |
| `run_stock_summary` | `pub async fn run_stock_summary(&mut self) -> CoreResult<PaginatedResponse<StockSummaryRow>>` | Fetch stock summary report from SyncServer (online). |
| `run_item_movement` | `pub async fn run_item_movement(&mut self) -> CoreResult<PaginatedResponse<ItemMovementRow>>` | Fetch item movement report from SyncServer (online). |

## Drafts

| Method | Signature | Description |
|---|---|---|
| `create_draft` | `pub async fn create_draft(&self, operation_type: OperationType, site_id: Option<i32>) -> CoreResult<OperationDraft>` | Create a new offline operation draft. Generates UUID, persists to SQLite. |
| `get_draft` | `pub async fn get_draft(&self, draft_id: &str) -> CoreResult<Option<OperationDraft>>` | Get draft by ID from local SQLite. |
| `list_drafts` | `pub async fn list_drafts(&self) -> CoreResult<Vec<OperationDraft>>` | List all drafts from local SQLite. |
| `delete_draft` | `pub async fn delete_draft(&self, draft_id: &str) -> CoreResult<()>` | Delete draft by ID from local SQLite. |
| `clone_draft` | `pub async fn clone_draft(&self, draft_id: &str) -> CoreResult<Option<OperationDraft>>` | Clone an existing draft (deep copy with new UUID). |
| `update_draft_header` | `pub async fn update_draft_header(&self, draft_id: &str, operation_type: Option<OperationType>, site_id: Option<Option<i32>>, effective_at: Option<Option<String>>, source_site_id: Option<Option<i32>>, destination_site_id: Option<Option<i32>>, recipient_id: Option<Option<i32>>, issued_to_name: Option<Option<String>>, comment: Option<Option<String>>) -> CoreResult<Option<OperationDraft>>` | Update draft header fields. Each field is `Option<Option<T>>` — `None` = skip, `Some(None)` = clear, `Some(Some(v))` = set. |
| `add_draft_item_line` | `pub async fn add_draft_item_line(&self, draft_id: &str, item_id: i32, qty: Value, batch: Option<String>, comment: Option<String>) -> CoreResult<Option<OperationDraft>>` | Add a catalog item line to a draft. `qty` is a JSON Value (supports decimal). |
| `add_draft_temp_item_line` | `pub async fn add_draft_temp_item_line(&self, draft_id: &str, temp_item: TemporaryItemInlineCreate, qty: Value, batch: Option<String>, comment: Option<String>) -> CoreResult<Option<OperationDraft>>` | Add a temporary item line to a receive-type draft. `temp_item` is created inline (not persisted separately). |
| `update_draft_line` | `pub async fn update_draft_line(&self, draft_id: &str, line_id: &str, qty: Option<Value>, batch: Option<Option<String>>, comment: Option<Option<String>>) -> CoreResult<Option<OperationDraft>>` | Update a draft line's qty, batch, and/or comment. |
| `delete_draft_line` | `pub async fn delete_draft_line(&self, draft_id: &str, line_id: &str) -> CoreResult<Option<OperationDraft>>` | Remove a line from a draft. |
| `validate_draft` | `pub async fn validate_draft(&self, draft_id: &str) -> Result<(), Vec<String>>` | Validate draft against business rules: required fields, positive quantities, temp-item constraints by operation type. Returns list of error strings or Ok(()). |

## Outbox

| Method | Signature | Description |
|---|---|---|
| `queue_draft_submit` | `pub async fn queue_draft_submit(&self, draft_id: &str) -> CoreResult<String>` | Validate draft, convert to operation create command, enqueue to outbox. Returns outbox event UUID. |
| `list_outbox_events` | `pub async fn list_outbox_events(&self, site_id: Option<i32>, status: Option<&str>) -> CoreResult<Vec<OutboxEvent>>` | List outbox events, optionally filtered by site_id and/or status. |
| `get_outbox_event` | `pub async fn get_outbox_event(&self, event_uuid: &str) -> CoreResult<Option<OutboxEvent>>` | Get single outbox event by UUID. |
| `retry_outbox_event` | `pub async fn retry_outbox_event(&self, event_uuid: &str) -> CoreResult<()>` | Reset a failed/cancelled outbox event back to pending for retry. |
| `cancel_outbox_event` | `pub async fn cancel_outbox_event(&self, event_uuid: &str) -> CoreResult<()>` | Mark an outbox event as cancelled. |
| `send_outbox` | `pub async fn send_outbox(&mut self) -> CoreResult<SendResult>` | Send pending outbox events to SyncServer (up to 10 per call). Returns `SendResult` with counts of accepted, rejected, and failed. |

## Sync Engine

| Method | Signature | Description |
|---|---|---|
| `sync_once` | `pub async fn sync_once(&mut self, mode: SyncMode) -> SyncResult` | Run full sync engine with 5 modes... |
| `cancel_sync` | `pub fn cancel_sync(&self)` | Request cancellation of in-progress sync. Sets `cancel_requested` flag and releases sync lock. |
| `is_syncing` | `pub fn is_syncing(&self) -> bool` | Check atomic sync lock. |
| `get_last_conflicts` | `pub fn get_last_conflicts(&self) -> ConflictSummary` | Return conflict summary from last `sync_once` call. |

## Online Mutations

| Method | Signature | Description |
|---|---|---|
| `accept_operation_lines` | `pub async fn accept_operation_lines(&mut self, operation_id: &str, request: &AcceptLinesRequest) -> CoreResult<OperationResponse>` | Accept pending operation lines on SyncServer (online). |
| `resolve_lost_asset` | `pub async fn resolve_lost_asset(&mut self, operation_line_id: i64, request: &LostAssetResolveRequest) -> CoreResult<Value>` | Resolve a lost asset record on SyncServer (online). |
| `delete_temp_item` | `pub async fn delete_temp_item(&mut self, temp_id: i32) -> CoreResult<()>` | Delete temporary item on SyncServer (online). |
| `approve_temp_item` | `pub async fn approve_temp_item(&mut self, temp_id: i32, request: &ApproveAsItemRequest) -> CoreResult<TemporaryItemDto>` | Approve temporary item as catalog item on SyncServer (online). |
| `merge_temp_item` | `pub async fn merge_temp_item(&mut self, temp_id: i32, request: &MergeToItemRequest) -> CoreResult<TemporaryItemDto>` | Merge temporary item into existing catalog item on SyncServer (online). |

## Issue Objects

| Method | Signature | Description |
|---|---|---|
| `list_issue_objects` | `pub async fn list_issue_objects(&mut self, page: u32, page_size: u32, search: Option<&str>) -> CoreResult<IssueObjectListResponse>` | List issue objects from SyncServer (online). Paginated, searchable. |
| `get_issue_object` | `pub async fn get_issue_object(&mut self, id: i32) -> CoreResult<IssueObjectDto>` | Get issue object by ID from SyncServer (online). |
| `create_issue_object` | `pub async fn create_issue_object(&mut self, body: &IssueObjectCreate) -> CoreResult<IssueObjectDto>` | Create issue object on SyncServer (online). |
| `update_issue_object` | `pub async fn update_issue_object(&mut self, id: i32, body: &IssueObjectUpdate) -> CoreResult<IssueObjectDto>` | Update issue object on SyncServer (online). |
| `delete_issue_object` | `pub async fn delete_issue_object(&mut self, id: i32) -> CoreResult<()>` | Delete issue object on SyncServer (online). |
| `merge_issue_objects` | `pub async fn merge_issue_objects(&mut self, body: &IssueObjectMerge) -> CoreResult<IssueObjectDto>` | Merge two issue objects on SyncServer (online). |
| `list_issue_object_assets` | `pub async fn list_issue_object_assets(&mut self, id: i32, page: u32, page_size: u32) -> CoreResult<PaginatedResponse<Value>>` | List assets belonging to an issue object (online). |
| `get_issue_object_tree` | `pub async fn get_issue_object_tree(&mut self) -> CoreResult<Vec<IssueObjectTreeDto>>` | Get issue object category tree from SyncServer (online). |
| `create_issue_object_category` | `pub async fn create_issue_object_category(&mut self, body: &IssueObjectCategoryCreate) -> CoreResult<IssueObjectCategoryDto>` | Create issue object category on SyncServer (online). |
| `list_issue_object_categories` | `pub async fn list_issue_object_categories(&mut self, page: u32, page_size: u32) -> CoreResult<PaginatedResponse<IssueObjectCategoryDto>>` | List issue object categories from SyncServer (online). |
| `get_issue_object_category` | `pub async fn get_issue_object_category(&mut self, id: i32) -> CoreResult<IssueObjectCategoryDto>` | Get issue object category by ID (online). |
| `update_issue_object_category` | `pub async fn update_issue_object_category(&mut self, id: i32, body: &IssueObjectCategoryUpdate) -> CoreResult<IssueObjectCategoryDto>` | Update issue object category on SyncServer (online). |
| `delete_issue_object_category` | `pub async fn delete_issue_object_category(&mut self, id: i32) -> CoreResult<()>` | Delete issue object category on SyncServer (online). |

---

## DTOs

### ProfileStatus

```rust
pub struct ProfileStatus {
    pub user_id: Uuid,
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
```

### AuthContextDto

```rust
pub struct AuthContextDto {
    pub user_id: Uuid,
    pub user_name: String,
    pub user_email: String,
    pub role: String,
    pub is_root: bool,
    pub available_sites: Vec<AuthSiteInfo>,
    pub device_id: i32,
    pub device_registered: bool,
}
```

### SyncStatusInfo

```rust
pub struct SyncStatusInfo {
    pub is_authenticated: bool,
    pub active_site_id: Option<i32>,
    pub outbox_pending: i64,
    pub cursor_count: i32,
}
```

### DiagnosticsInfo

```rust
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
```

### SyncResult / SyncRunSummary / ConflictSummary

```rust
pub struct SyncResult {
    pub error: Option<String>,
    pub mode: Option<SyncMode>,
    pub pull_summary: Option<SyncRunSummary>,
    pub push_summary: Option<SendResult>,
    pub conflicts: ConflictSummary,
}

pub struct SyncRunSummary {
    // Per-family success/failure counts
}

pub struct ConflictSummary {
    // Conflict details from last sync
}
```

---

## FFI Error Codes

| Code | Error | HTTP Equivalent |
|---|---|---|
| 0 | Success | — |
| 1 | Configuration error | — |
| 2 | I/O error | — |
| 3 | Database error | — |
| 4 | Network error | 500, connection refused |
| 5 | Authentication error | 401 |
| 6 | Serialization error | — |
| 7 | Validation error | 422 |
| 8 | Not found | 404 |
| 9 | Conflict | 409 |
| 10 | Sync error | — |
| 11 | Internal error | — |
| 12 | Forbidden | 403 |
| 13 | Timeout | 429, 504 |
| 14 | Unavailable | 503 |

## CoreConfig

```json
{
  "server_base_url": "http://localhost:8000",
  "database_path": "/path/to/warehouse.db",
  "client_name": "warehouse_client_core",
  "client_version": "0.3.0",
  "device_id": null,
  "site_id": null,
  "timeout_seconds": 30,
  "max_retries": 3,
  "retry_backoff_seconds": 2
}
```

## TokenProvider

- `NullTokenProvider` — no-op, returns `None` for both tokens.
- `CliTokenProvider` — reads `WAREHOUSE_USER_TOKEN` / `SYNC_USER_TOKEN` and `WAREHOUSE_DEVICE_TOKEN` / `SYNC_DEVICE_TOKEN` from environment variables.
- `FfiTokenProvider` — used by `warehouse_ffi`, tokens set via `ffi_set_user_token` / `ffi_set_device_token` extern functions.
- Custom: implement `TokenProvider` trait for platform secure storage (Android Keystore, Windows DPAPI).
