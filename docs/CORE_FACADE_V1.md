# CoreFacade V1 — Method / DTO / Error Reference

## Lifecycle

| Method | Signature | Description |
|---|---|---|
| `open` | `CoreConfig -> CoreResult<CoreHandle>` | Open profile with config |
| `open_readonly` | `&str -> CoreResult<CoreHandle>` | Open read-only SQLite |
| `close` | `()` | Release HTTP client |
| `set_token_provider` | `Box<dyn TokenProvider>` | Wire token provider |
| `config` | `-> &CoreConfig` | Current config |
| `profile_status` | `-> CoreResult<ProfileStatus>` | Current profile snapshot |
| `health_local` | `-> CoreResult<String>` | Local DB health check |
| `diagnostics` | `-> CoreResult<DiagnosticsInfo>` | Full diagnostics |

## Auth / Site

| Method | Signature | Description |
|---|---|---|
| `load_profile` | `-> CoreResult<()>` | Load from SQLite |
| `refresh_identity` | `-> CoreResult<()>` | GET /auth/context → save |
| `get_auth_context` | `-> CoreResult<AuthContextDto>` | Current auth context |
| `list_available_sites` | `-> CoreResult<Vec<AuthSiteInfo>>` | Available sites |
| `set_active_site` | `i32 -> CoreResult<()>` | Set active site |
| `get_active_site` | `-> CoreResult<Option<i32>>` | Current active site |
| `clear_active_site` | `-> CoreResult<()>` | Clear active site |
| `logout` | `-> CoreResult<()>` | Clear profile + client |

## Connectivity

| Method | Signature | Description |
|---|---|---|
| `health_remote` | `-> CoreResult<bool>` | GET /health |
| `ready_remote` | `-> CoreResult<String>` | GET /ready |

## Sync

| Method | Signature | Description |
|---|---|---|
| `bootstrap_device` | `-> CoreResult<Profile>` | POST /bootstrap/sync |
| `bootstrap` | `-> CoreResult<BootstrapResult>` | Multi-family bootstrap |
| `pull_once` | `-> SyncRunSummary` | Pull all 12 families |
| `get_sync_status` | `-> CoreResult<SyncStatusInfo>` | Cursors + outbox count |
| `list_sync_runs` | `-> CoreResult<Vec<SyncRunSummary>>` | History |
| `sync_once` | `SyncMode -> SyncResult` | Full sync engine run |

## Catalog

| Method | Signature | Description |
|---|---|---|
| `search_items` | `&str -> CoreResult<Vec<ItemDto>>` | Local SQLite search |
| `get_item` | `i32 -> CoreResult<Option<ItemDto>>` | By ID |
| `list_units` | `-> CoreResult<Vec<UnitDto>>` | All units |
| `list_categories` | `-> CoreResult<Vec<CategoryDto>>` | All categories |
| `get_category_tree` | `-> CoreResult<Vec<CategoryTreeNode>>` | Recursive tree |
| `get_parent_chain` | `i32 -> CoreResult<Vec<CategoryDto>>` | Ancestry chain |

## Balances / Assets

| Method | Signature | Description |
|---|---|---|
| `list_balances` | `i32 -> CoreResult<Vec<BalanceRow>>` | By site |
| `get_balance_by_subject` | `i32 -> CoreResult<Vec<BalanceRow>>` | By item |
| `get_balance_summary` | `-> CoreResult<PaginatedResponse<BalanceSummaryRow>>` | Online |
| `list_pending_acceptance` | `-> CoreResult<Vec<PendingAcceptanceRow>>` | Local cache |
| `list_lost_assets` | `-> CoreResult<Vec<LostAssetRow>>` | Local cache |
| `list_issued_assets` | `-> CoreResult<Vec<IssuedAssetRow>>` | Local cache |

## Operations

| Method | Signature | Description |
|---|---|---|
| `list_operations` | `(i32, u32, u32) -> CoreResult<PaginatedResponse<OperationListItem>>` | Online |
| `get_operation` | `i64 -> CoreResult<OperationResponse>` | Online |

## Recipients

| Method | Signature | Description |
|---|---|---|
| `search_recipients` | `&str -> CoreResult<Vec<RecipientDto>>` | Local search |
| `get_recipient` | `i32 -> CoreResult<Option<RecipientDto>>` | By ID |

## Temporary Items

| Method | Signature | Description |
|---|---|---|
| `list_temporary_items` | `-> CoreResult<Vec<TemporaryItemDto>>` | Local cache |
| `get_temporary_item` | `i32 -> CoreResult<Option<TemporaryItemDto>>` | By ID |
| `list_temporary_item_operations` | `(i32) -> ...` | Online |
| `approve_temp_item` | `(i32, &ApproveAsItemRequest) -> CoreResult<TemporaryItemDto>` | Online |
| `merge_temp_item` | `(i32, &MergeToItemRequest) -> CoreResult<TemporaryItemDto>` | Online |
| `delete_temp_item` | `i32 -> CoreResult<()>` | Online |

## Documents / Reports

| Method | Signature | Description |
|---|---|---|
| `list_documents` | `i32 -> CoreResult<Vec<DocumentDto>>` | Local cache |
| `get_document` | `&str -> CoreResult<Option<DocumentDto>>` | By ID |
| `list_operation_documents` | `Uuid -> CoreResult<Vec<DocumentDto>>` | Online |
| `generate_document` | `&DocumentGenerateRequest -> CoreResult<DocumentDto>` | Online |
| `render_document` | `Uuid -> CoreResult<Vec<u8>>` | Online (raw bytes) |
| `run_stock_summary` | `-> CoreResult<PaginatedResponse<StockSummaryRow>>` | Online |
| `run_item_movement` | `-> CoreResult<PaginatedResponse<ItemMovementRow>>` | Online |

## Drafts

| Method | Signature | Description |
|---|---|---|
| `create_draft` | `(OperationType, Option<i32>) -> CoreResult<OperationDraft>` | Local |
| `get_draft` | `&str -> CoreResult<Option<OperationDraft>>` | By ID |
| `list_drafts` | `-> CoreResult<Vec<OperationDraft>>` | All |
| `delete_draft` | `&str -> CoreResult<()>` | By ID |
| `clone_draft` | `&str -> CoreResult<Option<OperationDraft>>` | Copy |
| `update_draft_header` | `(&str, Option<OperationType>, ...) -> CoreResult<Option<OperationDraft>>` | 9 fields |
| `add_draft_item_line` | `(&str, i32, Value, Option<String>, Option<String>) -> ...` | Add catalog item line |
| `add_draft_temp_item_line` | `(&str, TemporaryItemInlineCreate, Value, ...) -> ...` | Add temp item line |
| `update_draft_line` | `(&str, &str, Option<Value>, Option<Option<String>>, ...) -> ...` | Update line |
| `delete_draft_line` | `(&str, &str) -> ...` | Remove line |
| `validate_draft` | `&str -> Result<(), Vec<String>>` | Validate |

## Outbox

| Method | Signature | Description |
|---|---|---|
| `queue_draft_submit` | `&str -> CoreResult<String>` | Validate + enqueue |
| `list_outbox_events` | `(Option<i32>, Option<&str>) -> CoreResult<Vec<OutboxEvent>>` | Filter |
| `get_outbox_event` | `&str -> CoreResult<Option<OutboxEvent>>` | By UUID |
| `retry_outbox_event` | `&str -> CoreResult<()>` | Reset to pending |
| `cancel_outbox_event` | `&str -> CoreResult<()>` | Mark cancelled |
| `send_outbox` | `-> CoreResult<SendResult>` | Send pending |

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
  "client_version": "0.1.0",
  "device_id": null,
  "site_id": null,
  "timeout_seconds": 30,
  "max_retries": 3,
  "retry_backoff_seconds": 2
}
```

## TokenProvider

- `NullTokenProvider` — no-op, returns None
- `CliTokenProvider` — reads `WAREHOUSE_USER_TOKEN` / `SYNC_USER_TOKEN` and `WAREHOUSE_DEVICE_TOKEN` / `SYNC_DEVICE_TOKEN` from env
- Custom: implement `TokenProvider` trait for platform secure storage
