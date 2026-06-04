# Client-Ready API Endpoint Coverage Matrix

> Source: `SyncServer` OpenAPI spec (live) + `API_MAP.md` + `crates/warehouse_core/src/syncserver/*.rs`
> Base path: `/api/v1`
> Generated: 2026-06-04

## Legend

| Column | Meaning |
|---|---|
| **Core Client** | Method in `crates/warehouse_core/src/syncserver/<module>.rs` |
| **Facade** | Method in `crates/warehouse_core/src/facade/mod.rs` that delegates to HTTP client |
| **FFI Export** | `extern "C" fn` in `crates/warehouse_ffi/src/lib.rs` |
| ✅ | Implemented |
| ✅ (fg) | Feature-gated (`admin-api` feature) |
| ✅ (local) | Facade reads from local SQLite (populated by sync), not direct HTTP |
| ❌ | Not implemented |

---

## 1. Health API (5 endpoints)

| # | Path | Verb | Core Client | Facade | FFI Export | Status |
|---|---|---|---|---|---|---|
| 1 | `/` | GET | `server_info()` | ❌ | ❌ | ❌ |
| 2 | `/health` | GET | `health()` | `health_remote()` | ❌ | ✅ |
| 3 | `/ready` | GET | `ready()` | `ready_remote()` | `warehouse_ready_remote` | ✅ |
| 4 | `/health/detailed` | GET | `health_detailed()` | ❌ | ❌ | ✅ (client only) |
| 5 | `/health/liveness` | GET | `health_liveness()` | ❌ | ❌ | ✅ (client only) |
| 6 | `/health/readiness` | GET | ❌ | ❌ | ❌ | ❌ |

---

## 2. Auth API (4 endpoints)

| # | Path | Verb | Core Client | Facade | FFI Export | Status |
|---|---|---|---|---|---|---|
| 7 | `/auth/sync-user` | POST | `auth_sync_user()` | ❌ | ❌ | ✅ (client only) |
| 8 | `/auth/me` | GET | `auth_me()` | ❌ | ❌ | ✅ (client only) |
| 9 | `/auth/sites` | GET | `auth_sites()` | ❌ (uses local profile) | ❌ | ✅ (client only) |
| 10 | `/auth/context` | GET | `auth_context()` | `refresh_identity()` | `warehouse_refresh_identity` | ✅ |

---

## 3. Catalog — Sync Read (5 endpoints)

Data pulled via `/catalog/items`, `/catalog/categories`, `/catalog/units`, `/catalog/sites` into local SQLite; facade reads from local repos. Browse/search endpoints use local data (not direct HTTP).

| # | Path | Verb | Core Client | Facade | FFI Export | Status |
|---|---|---|---|---|---|---|
| 11 | `/catalog/items` | GET | `catalog_items()` | ❌ (local) | ❌ | ✅ (client only) |
| 12 | `/catalog/categories` | GET | `catalog_categories()` | ❌ (local) | ❌ | ✅ (client only) |
| 13 | `/catalog/categories/tree` | GET | `catalog_category_tree()` | `get_category_tree()` (local) | ❌ | ✅ |
| 14 | `/catalog/units` | GET | `catalog_units()` | `list_units()` (local) | ❌ | ✅ |
| 15 | `/catalog/sites` | GET | `catalog_sites()` | ❌ (local profile) | ❌ | ✅ (client only) |

---

## 4. Catalog — Browse Read (5 endpoints)

These are search/browse endpoints available in the OpenAPI spec. The Rust client does NOT implement them — instead the facade reads from local SQLite (populated by pull-sync).

| # | Path | Verb | Core Client | Facade | FFI Export | Status |
|---|---|---|---|---|---|---|
| 16 | `/catalog/read/items` | GET | ❌ | `search_items()` (local) | `warehouse_search_catalog` | ❌ (HTTP) / ✅ (local) |
| 17 | `/catalog/read/categories` | GET | ❌ | `list_categories()` (local) | ❌ | ❌ (HTTP) / ✅ (local) |
| 18 | `/catalog/read/categories/{id}/items` | GET | ❌ | ❌ | ❌ | ❌ |
| 19 | `/catalog/read/categories/{id}/children` | GET | ❌ | ❌ | ❌ | ❌ |
| 20 | `/catalog/read/categories/{id}/parent-chain` | GET | ❌ | `get_parent_chain()` (local) | ❌ | ❌ (HTTP) / ✅ (local) |

---

## 5. Catalog Admin API (15 endpoints) — REQUIRED-ADMIN

Feature-gated behind `#[cfg(feature = "admin-api")]`. Only `categories_create_bulk` and `units_create_bulk` are implemented in the HTTP client. Individual CRUD for units/categories/items is **NOT** implemented in core client.

| # | Path | Verb | Core Client | Facade | FFI Export | Status |
|---|---|---|---|---|---|---|
| 21 | `/catalog/admin/units` | GET | ❌ | ❌ | ❌ | ❌ |
| 22 | `/catalog/admin/units/{id}` | GET | ❌ | ❌ | ❌ | ❌ |
| 23 | `/catalog/admin/units` | POST | ❌ | ❌ | ❌ | ❌ |
| 24 | `/catalog/admin/units/bulk` | POST | `units_create_bulk()` | ❌ | ❌ | ✅ (fg, client only) |
| 25 | `/catalog/admin/units/{id}` | PATCH | ❌ | ❌ | ❌ | ❌ |
| 26 | `/catalog/admin/units/{id}` | DELETE | ❌ | ❌ | ❌ | ❌ |
| 27 | `/catalog/admin/categories` | GET | ❌ | ❌ | ❌ | ❌ |
| 28 | `/catalog/admin/categories/{id}` | GET | ❌ | ❌ | ❌ | ❌ |
| 29 | `/catalog/admin/categories` | POST | ❌ | ❌ | ❌ | ❌ |
| 30 | `/catalog/admin/categories/bulk` | POST | `categories_create_bulk()` | ❌ | ❌ | ✅ (fg, client only) |
| 31 | `/catalog/admin/categories/{id}` | PATCH | ❌ | ❌ | ❌ | ❌ |
| 32 | `/catalog/admin/categories/{id}` | DELETE | ❌ | ❌ | ❌ | ❌ |
| 33 | `/catalog/admin/items` | GET | ❌ | ❌ | ❌ | ❌ |
| 34 | `/catalog/admin/items/{id}` | GET | ❌ | ❌ | ❌ | ❌ |
| 35 | `/catalog/admin/items` | POST | ❌ | ❌ | ❌ | ❌ |
| 36 | `/catalog/admin/items/{id}` | PATCH | ❌ | ❌ | ❌ | ❌ |
| 37 | `/catalog/admin/items/{id}` | DELETE | ❌ | ❌ | ❌ | ❌ |
| 38 | `/catalog/admin/batch` | POST | ❌ | ❌ | ❌ | ❌ |

---

## 6. Operations API (9 endpoints)

| # | Path | Verb | Core Client | Facade | FFI Export | Status |
|---|---|---|---|---|---|---|
| 39 | `/operations` | GET | `operations_list()` | `list_operations()` | `warehouse_list_operations` | ✅ |
| 40 | `/operations/{id}` | GET | `operations_get()` | `get_operation()` | ❌ | ✅ (facade) |
| 41 | `/operations` | POST | `operations_create()` / `create_operation_raw()` | via outbox (`send_outbox`) | via outbox FFI | ✅ |
| 42 | `/operations/{id}` | PATCH | `operations_update()` | ❌ | ❌ | ✅ (client only) |
| 43 | `/operations/{id}/effective-at` | PATCH | `operations_set_effective_at()` | ❌ | ❌ | ✅ (client only) |
| 44 | `/operations/{id}/submit` | POST | `operations_submit()` | via outbox | via outbox FFI | ✅ (client only) |
| 45 | `/operations/{id}/cancel` | POST | `operations_cancel()` | ❌ | ❌ | ✅ (client only) |
| 46 | `/operations/{id}/accept-lines` | POST | `operations_accept_lines()` | `accept_operation_lines()` | ❌ | ✅ (facade) |
| 47 | `/operations/{id}` | DELETE | `operations_delete()` | `delete_operation()` | ❌ | ✅ (facade) |

---

## 7. Balances API (3 endpoints)

Balances are sync-pulled into local SQLite. `balances_summary` goes directly to server.

| # | Path | Verb | Core Client | Facade | FFI Export | Status |
|---|---|---|---|---|---|---|
| 48 | `/balances` | GET | `balances_list()` | `list_balances()` (local) | `warehouse_list_balances` | ✅ |
| 49 | `/balances/by-site` | GET | `balances_by_site()` | ❌ (local) | ❌ | ✅ (client only) |
| 50 | `/balances/summary` | GET | `balances_summary()` | `get_balance_summary()` | ❌ | ✅ (facade) |

---

## 8. Temporary Items API (6 endpoints)

| # | Path | Verb | Core Client | Facade | FFI Export | Status |
|---|---|---|---|---|---|---|
| 51 | `/temporary-items` | GET | `temporary_items_list()` | `list_temporary_items()` (local) | `warehouse_list_temp_items` | ✅ |
| 52 | `/temporary-items/{id}` | GET | `temporary_items_get()` | `get_temporary_item()` (local) | ❌ | ✅ |
| 53 | `/temporary-items/{id}/approve-as-item` | POST | `temporary_items_approve()` | `approve_temp_item()` | `warehouse_approve_temp_item` | ✅ |
| 54 | `/temporary-items/{id}/merge` | POST | `temporary_items_merge()` | `merge_temp_item()` | `warehouse_merge_temp_item` | ✅ |
| 55 | `/temporary-items/{id}/operations` | GET | `temporary_items_operations()` | `list_temporary_item_operations()` | ❌ | ✅ |
| 56 | `/temporary-items/{id}` | DELETE | `temporary_items_delete()` | `delete_temp_item()` | `warehouse_delete_temp_item` | ✅ |

---

## 9. Documents API (7 endpoints)

| # | Path | Verb | Core Client | Facade | FFI Export | Status |
|---|---|---|---|---|---|---|
| 57 | `/documents/generate` | POST | `documents_generate()` | `generate_document()` | `warehouse_generate_document` | ✅ |
| 58 | `/documents/{id}` | GET | `documents_get()` | `get_document()` (local) | ❌ | ✅ |
| 59 | `/documents/{id}/render` | GET | `documents_render()` | `render_document()` | `warehouse_render_document` | ✅ |
| 60 | `/documents` | GET | `documents_list()` | `list_documents()` (local) | ❌ | ✅ |
| 61 | `/documents/{id}/status` | PATCH | `documents_update_status()` | ❌ | ❌ | ✅ (client only) |
| 62 | `/documents/operations/{op_id}/documents` | GET | `documents_for_operation()` | `list_operation_documents()` | ❌ | ✅ |
| 63 | `/documents/operations/{op_id}/documents` | POST | `documents_create_for_operation()` | ❌ | ❌ | ✅ (client only) |

---

## 10. Recipients API (6 endpoints) — LEGACY

> **API_MAP.md note:** Recipient model has been removed and replaced by Issue Objects API. The core client still has a `recipients` module for backwards compatibility. New clients should use Issue Objects instead.

| # | Path | Verb | Core Client | Facade | FFI Export | Status |
|---|---|---|---|---|---|---|
| 64 | `/recipients` | GET | `recipients_list()` | `search_recipients()` (local) | `warehouse_search_recipients` | ✅ |
| 65 | `/recipients/{id}` | GET | `recipients_get()` | `get_recipient()` (local) | `warehouse_get_recipient` | ✅ |
| 66 | `/recipients` | POST | `recipients_create()` | ❌ | ❌ | ✅ (client only) |
| 67 | `/recipients/{id}` | PATCH | `recipients_update()` | ❌ | ❌ | ✅ (client only) |
| 68 | `/recipients/{id}` | DELETE | `recipients_delete()` | ❌ | ❌ | ✅ (client only) |
| 69 | `/recipients/merge` | POST | `recipients_merge()` | ❌ | ❌ | ✅ (client only) |

---

## 11. Asset Registers API (5 endpoints)

| # | Path | Verb | Core Client | Facade | FFI Export | Status |
|---|---|---|---|---|---|---|
| 70 | `/pending-acceptance` | GET | `pending_acceptance_list()` | `list_pending_acceptance()` (local) | `warehouse_list_pending_acceptance` | ✅ |
| 71 | `/lost-assets` | GET | `lost_assets_list()` | `list_lost_assets()` (local) | `warehouse_list_lost_assets` | ✅ |
| 72 | `/lost-assets/{line_id}` | GET | `lost_assets_get()` | ❌ | ❌ | ✅ (client only) |
| 73 | `/lost-assets/{line_id}/resolve` | POST | `lost_assets_resolve()` | `resolve_lost_asset()` | `warehouse_resolve_lost_asset` | ✅ |
| 74 | `/issued-assets` | GET | `issued_assets_list()` | `list_issued_assets()` (local) | `warehouse_list_issued_assets` | ✅ |

---

## 12. Reports API (2 endpoints)

| # | Path | Verb | Core Client | Facade | FFI Export | Status |
|---|---|---|---|---|---|---|
| 75 | `/reports/item-movement` | GET | `reports_item_movement()` | `run_item_movement()` | `warehouse_item_movement` | ✅ |
| 76 | `/reports/stock-summary` | GET | `reports_stock_summary()` | `run_stock_summary()` | `warehouse_stock_summary` | ✅ |

---

## 13. Issue Objects API (14 endpoints)

Replaces the old Recipients API. Full set of CRUD + merge + tree + asset listing for issue objects and their categories.

| # | Path | Verb | Core Client | Facade | FFI Export | Status |
|---|---|---|---|---|---|---|
| 77 | `/issue-objects` | GET | `issue_objects_list()` | `list_issue_objects()` | `warehouse_list_issue_objects` | ✅ |
| 78 | `/issue-objects` | POST | `issue_objects_create()` | `create_issue_object()` | `warehouse_create_issue_object` | ✅ |
| 79 | `/issue-objects/{id}` | GET | `issue_objects_get()` | `get_issue_object()` | `warehouse_get_issue_object` | ✅ |
| 80 | `/issue-objects/{id}` | PATCH | `issue_objects_update()` | `update_issue_object()` | `warehouse_update_issue_object` | ✅ |
| 81 | `/issue-objects/{id}` | DELETE | `issue_objects_delete()` | `delete_issue_object()` | `warehouse_delete_issue_object` | ✅ |
| 82 | `/issue-objects/merge` | POST | `issue_objects_merge()` | `merge_issue_objects()` | `warehouse_merge_issue_objects` | ✅ |
| 83 | `/issue-objects/{id}/assets` | GET | `issue_objects_list_assets()` | `list_issue_object_assets()` | `warehouse_list_issue_object_assets` | ✅ |
| 84 | `/issue-objects/tree` | GET | `issue_objects_get_tree()` | `get_issue_object_tree()` | `warehouse_get_issue_object_tree` | ✅ |
| 85 | `/issue-object-categories` | POST | `issue_object_categories_create()` | `create_issue_object_category()` | `warehouse_create_issue_object_category` | ✅ |
| 86 | `/issue-object-categories` | GET | `issue_object_categories_list()` | `list_issue_object_categories()` | `warehouse_list_issue_object_categories` | ✅ |
| 87 | `/issue-object-categories/{id}` | GET | `issue_object_categories_get()` | `get_issue_object_category()` | `warehouse_get_issue_object_category` | ✅ |
| 88 | `/issue-object-categories/{id}` | PATCH | `issue_object_categories_update()` | `update_issue_object_category()` | `warehouse_update_issue_object_category` | ✅ |
| 89 | `/issue-object-categories/{id}` | DELETE | `issue_object_categories_delete()` | `delete_issue_object_category()` | `warehouse_delete_issue_object_category` | ✅ |

---

## 14. Device Sync API (4 endpoints)

Device sync endpoints are used internally by the sync engine, not directly exposed through facade/FFI (except bootstrap).

| # | Path | Verb | Core Client | Facade | FFI Export | Status |
|---|---|---|---|---|---|---|
| 90 | `/ping` | POST | `ping()` | ❌ (sync engine internal) | ❌ | ✅ (client only) |
| 91 | `/push` | POST | `push()` | ❌ (sync engine internal) | ❌ | ✅ (client only) |
| 92 | `/pull` | POST | `pull()` | ❌ (sync engine internal) | ❌ | ✅ (client only) |
| 93 | `/bootstrap/sync` | POST | `bootstrap_sync()` | `bootstrap_device()` / `bootstrap()` | `warehouse_bootstrap` | ✅ |

---

## 15. Admin API (18 endpoints) — REQUIRED-ADMIN

Feature-gated behind `#[cfg(feature = "admin-api")]`. Admin client methods exist but are NOT exposed through facade or FFI.

### Sites (3)

| # | Path | Verb | Core Client | Facade | FFI Export | Status |
|---|---|---|---|---|---|---|
| 94 | `/admin/sites` | GET | `admin_sites_list()` | ❌ | ❌ | ✅ (fg, client only) |
| 95 | `/admin/sites` | POST | `admin_sites_create()` | ❌ | ❌ | ✅ (fg, client only) |
| 96 | `/admin/sites/{id}` | PATCH | `admin_sites_update()` | ❌ | ❌ | ✅ (fg, client only) |

### Users (6 + 3 sub-resources)

| # | Path | Verb | Core Client | Facade | FFI Export | Status |
|---|---|---|---|---|---|---|
| 97 | `/admin/users` | GET | `admin_users_list()` | ❌ | ❌ | ✅ (fg, client only) |
| 98 | `/admin/users/{id}` | GET | `admin_users_get()` | ❌ | ❌ | ✅ (fg, client only) |
| 99 | `/admin/users` | POST | `admin_users_create()` | ❌ | ❌ | ✅ (fg, client only) |
| 100 | `/admin/users/{id}` | PATCH | `admin_users_update()` | ❌ | ❌ | ✅ (fg, client only) |
| 101 | `/admin/users/{id}` | DELETE | `admin_users_delete()` | ❌ | ❌ | ✅ (fg, client only) |
| 102 | `/admin/users/{id}/rotate-token` | POST | `admin_users_rotate_token()` | ❌ | ❌ | ✅ (fg, client only) |
| 103 | `/admin/users/{id}/sync-state` | GET | ❌ | ❌ | ❌ | ❌ |
| 104 | `/admin/users/{id}/scopes` | PUT | ❌ | ❌ | ❌ | ❌ |

### Devices (6 + 1 sub-resource)

| # | Path | Verb | Core Client | Facade | FFI Export | Status |
|---|---|---|---|---|---|---|
| 105 | `/admin/devices` | GET | `admin_devices_list()` | ❌ | ❌ | ✅ (fg, client only) |
| 106 | `/admin/devices/{id}` | GET | `admin_devices_get()` | ❌ | ❌ | ✅ (fg, client only) |
| 107 | `/admin/devices` | POST | `admin_devices_register()` | ❌ | ❌ | ✅ (fg, client only) |
| 108 | `/admin/devices/{id}` | PATCH | `admin_devices_update()` | ❌ | ❌ | ✅ (fg, client only) |
| 109 | `/admin/devices/{id}` | DELETE | `admin_devices_delete()` | ❌ | ❌ | ✅ (fg, client only) |
| 110 | `/admin/devices/{id}/rotate-token` | POST | `admin_devices_rotate_token()` | ❌ | ❌ | ✅ (fg, client only) |

### Access Scopes (3)

| # | Path | Verb | Core Client | Facade | FFI Export | Status |
|---|---|---|---|---|---|---|
| 111 | `/admin/access/scopes` | GET | ❌ | ❌ | ❌ | ❌ |
| 112 | `/admin/access/scopes` | POST | ❌ | ❌ | ❌ | ❌ |
| 113 | `/admin/access/scopes/{id}` | PATCH | ❌ | ❌ | ❌ | ❌ |

### Roles

| # | Path | Verb | Core Client | Facade | FFI Export | Status |
|---|---|---|---|---|---|---|
| 114 | `/admin/roles` | GET | ❌ | ❌ | ❌ | ❌ |

---

## 16. Review Items API (6 endpoints) — NOT YET INDEXED

> **New group discovered in OpenAPI spec** — not present in `API_MAP.md` or old matrix.

| # | Path | Verb | Core Client | Facade | FFI Export | Status |
|---|---|---|---|---|---|---|
| 115 | `/review-items` | GET | ❌ | ❌ | ❌ | ❌ |
| 116 | `/review-items/{id}` | GET | ❌ | ❌ | ❌ | ❌ |
| 117 | `/review-items/{id}/operations` | GET | ❌ | ❌ | ❌ | ❌ |
| 118 | `/review-items/{id}/confirm` | POST | ❌ | ❌ | ❌ | ❌ |
| 119 | `/review-items/{id}/merge` | POST | ❌ | ❌ | ❌ | ❌ |
| 120 | `/review-items/{id}` | DELETE | ❌ | ❌ | ❌ | ❌ |

---

## Summary

| Category | Total | Client Only | Facade + Client | FFI Export | Missing |
|---|---|---|---|---|---|
| Health (incl. `/`) | 6 | 2 | 2 | 1 | 2 |
| Auth | 4 | 3 | 1 | 1 | 0 |
| Catalog — Sync Read | 5 | 2 | 3 | 0 | 0 |
| Catalog — Browse Read | 5 | 0 | 0 | 0 | 5 |
| Catalog Admin | 18 | 2 | 0 | 0 | 16 |
| Operations | 9 | 4 | 5 | 1 | 0 |
| Balances | 3 | 1 | 2 | 1 | 0 |
| Temporary Items | 6 | 0 | 6 | 3 | 0 |
| Documents | 7 | 2 | 5 | 2 | 0 |
| Recipients (legacy) | 6 | 4 | 2 | 2 | 0 |
| Asset Registers | 5 | 1 | 4 | 4 | 0 |
| Reports | 2 | 0 | 2 | 2 | 0 |
| Issue Objects | 13 | 0 | 13 | 13 | 0 |
| Device Sync | 4 | 3 | 1 | 1 | 0 |
| Admin | 21 | 11 | 0 | 0 | 10 |
| Review Items | 6 | 0 | 0 | 0 | 6 |
| **Total** | **120** | **35** | **46** | **31** | **39** |

- **Facade + FFI Ready (full stack):** 31 endpoints — HTTP client → Facade → FFI export
- **Facade Only (no FFI):** 15 endpoints — HTTP client → Facade, but no FFI export
- **Client Only (no facade/FFI):** 35 endpoints — HTTP client method exists, but not exposed through Facade/FFI
- **Missing:** 39 endpoints

## Key Observations

1. **Issue Objects API is fully implemented** — all 13 endpoints have HTTP client, facade, and FFI exports. Best coverage in the project.
2. **Reports and Asset Registers** have good coverage with all endpoints wired through to FFI.
3. **Operations API** has all 9 endpoints in the HTTP client, but only `list_operations` has a complete facade→FFI path.
4. **Admin API** is feature-gated (`admin-api` feature) and has no facade or FFI exposure.
5. **Browse catalog read endpoints** are not implemented as HTTP calls — the front-end reads from local SQLite populated by pull-sync.
6. **Review Items API** is a new group discovered in the live OpenAPI spec with no client implementation at all.
7. **Recipients API** is legacy — the server replaced it with Issue Objects, but the core client still has full HTTP coverage.
8. **Missing endpoints from OpenAPI:** `/admin/roles`, `/admin/access/scopes` (CRUD), `/admin/users/{id}/sync-state`, `/admin/users/{id}/scopes`, all 5 browse catalog endpoints, `/health/readiness`, all 6 review items, `/catalog/admin/batch`.
