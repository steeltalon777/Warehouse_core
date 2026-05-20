# TZ: Warehouse_client_core Stand Contract Fixes — Orchestrator Master Plan

## Execution Checklist

- [x] 0. Context verified
- [x] 1. Architecture boundaries confirmed
- [x] 2. Fix A: SQLite first-run DB file creation (create missing directory + file)
- [x] 3. Fix B: `SqliteDraftRepo::save()` binding bug (11 placeholders, 4 binds)
- [x] 4. Fix C: `write_sites()` missing `updated_at` NOT NULL
- [x] 5. Fix D: Bootstrap catalog write order (categories → units → items)
- [x] 6. Fix E: Operation/asset DTO IDs: `i64` → `String`/`Uuid` (SyncServer returns UUID)
- [x] 7. Fix F: Sync device_id: `Uuid` → `i32` (SyncServer expects integer device_id)
- [x] 8. Fix G: `BalanceRow.site_code` required but API returns `site_name`
- [x] 9. Fix H: Page sizes capped to server limits
- [x] 10. Fix I: `documents_list` response shape mismatch
- [x] 11. Fix J: Pull failures preserve family name (not `"unknown"`)
- [x] 12. Fix K: Bootstrap/sync engine returns failure on required-family errors
- [x] 13. Static checks (fmt, clippy, test) — all pass
- [x] 14. Unit/component tests: 40/40 pass
- [ ] 15. Integration tests with real dependencies — **2026-05-20 stand run executed; failed: bootstrap `catalog_items` FK, sync-pull serialization/page-size/FK errors**
- [ ] 16. Stand smoke tests — full green gate — **2026-05-20 partial pass only: health/auth/site/draft OK; bootstrap/sync/read not green**
- [x] 17. Documentation updated (reports + TZ status)
- [ ] 18. Final acceptance review — **blocked by failed stand smoke**

## Check Rules

- Architect creates this checklist and acceptance criteria.
- Executor agents may check boxes only after implementation AND verification are complete.
- QA verifier may check final acceptance only after evidence review.
- Failed or unavailable checks stay unchecked with a blocker note.
- **Order matters**: Fixes A through K are listed in dependency order. Do not skip ahead.

---

## 0. Purpose

This is the **orchestrator master plan** for fixing all 11 Core ↔ SyncServer contract and persistence bugs discovered in the real stand smoke test (2026-05-19, `CORE_STAND_SMOKE_REPORT.md`).

After all fixes A–K are complete and stand smoke passes green, the Core will be **Client-Ready** for read-only and basic offline-draft scenarios. The WPF `WarehouseAIWorkstation` migration gate will then be open.

**Predecessor documents:**
- `CORE_STAND_SMOKE_REPORT.md` — bug inventory
- `TZ_CORE_STAND_CONTRACT_FIXES_BEFORE_AIWORKSTATION.md` — higher-level scope/Boundaries
- `TZ_CORE_CLIENT_READY_COMPLETION.md` — overall roadmap (Levels 4-5 blocked)

**This TZ** is the concrete, line-level implementation plan.

---

## 1. Architecture Boundaries (re-confirmed)

- SyncServer is the source of truth. Core DTOs must match SyncServer wire format.
- Core owns local SQLite schema, outbox, sync, DTO mapping.
- No SyncServer schema changes are assumed or requested.
- No WPF/AIWorkstation runtime code is touched.

---

## 2. Fix A — SQLite First-Run DB File Creation

**Bug:** `CoreHandle::open` / CLI `db init` fails when `<profile>/warehouse.db` does not exist. Requires manual `New-Item` of empty file first.

**Root cause:** `storage/mod.rs` → `sqlite_url()` returns raw path (not `sqlite://` URI). `SqlitePool::connect()` may not auto-create files with raw paths on Windows.

**Files to change:**
- `crates/warehouse_core/src/storage/mod.rs`

**Fix approach:**
1. In `Database::open()`, before `SqlitePool::connect()`, check if DB file exists.
2. If not, create the file (and parent directories) with `std::fs::File::create()`.
3. Use `format!("sqlite:///{db_path}")` URI for sqlx connect, handling Windows path separators.

**Verification:**
```powershell
Remove-Item -Force "$env:LOCALAPPDATA\warehouse_client_core\warehouse.db" -ErrorAction SilentlyContinue
cargo run -p warehouse_cli -- db init
# Must succeed without manual file creation
```

**Acceptance:** Empty profile directory → `db init` succeeds → `db info` shows version 4.

---

## 3. Fix B — `SqliteDraftRepo::save()` Binding Bug

**Bug:** `draft create RECEIVE --site-id 1` fails with `NOT NULL constraint failed: operation_drafts.operation_type`.

**Root cause:** `repos.rs` lines 787-798: INSERT has 11 `?` placeholders but only 4 `.bind()` calls. The values are bound to wrong columns:
- `draft.comment` → bound as `operation_type` (column 2)
- `draft.created_at` → bound as `site_id` (column 3)
- `draft.updated_at` → bound as `effective_at` (column 4)
- Columns 5–11 (source_site_id, destination_site_id, recipient_id, issued_to_name, comment, created_at, updated_at) receive **no value** → NULL

**File to change:**
- `crates/warehouse_core/src/storage/repos.rs`, lines 784-799

**Fix approach:** Add the missing 7 `.bind()` calls. Correct bind order:

```rust
.bind(draft.draft_id.to_string())               // 1: draft_id
.bind(serde_json::to_string(&draft.operation_type).unwrap_or_default()) // 2: operation_type
.bind(draft.site_id)                             // 3: site_id
.bind(&draft.effective_at)                       // 4: effective_at
.bind(draft.source_site_id)                      // 5: source_site_id
.bind(draft.destination_site_id)                 // 6: destination_site_id
.bind(draft.recipient_id)                        // 7: recipient_id
.bind(&draft.issued_to_name)                     // 8: issued_to_name
.bind(&draft.comment)                            // 9: comment
.bind(&draft.created_at)                         // 10: created_at
.bind(&draft.updated_at)                         // 11: updated_at
```

**Verification:**
```powershell
cargo run -p warehouse_cli -- draft create RECEIVE --site-id 1
cargo run -p warehouse_cli -- draft list
cargo run -p warehouse_cli -- draft get <draft-id>
```

**Acceptance:** Draft create succeeds. Draft list shows the created draft. Operation type is `RECEIVE`.

---

## 4. Fix C — `write_sites()` Missing `updated_at`

**Bug:** Bootstrap/sync-pull fails with `NOT NULL constraint failed: sites.updated_at`.

**Root cause:** `snapshot_writer.rs` line 127: INSERT only includes `site_id, code, name, is_active`. SQLite schema (migration 0002 line 19): `updated_at TEXT NOT NULL`.

**File to change:**
- `crates/warehouse_core/src/storage/snapshot_writer.rs`, lines 116-140

**Fix approach:** Add `updated_at` to the INSERT, using `now_str()`:

```rust
sqlx::query("INSERT INTO sites (site_id, code, name, is_active, updated_at) VALUES (?, ?, ?, ?, ?)")
    .bind(s.site_id)
    .bind(&s.code)
    .bind(&s.name)
    .bind(s.is_active)
    .bind(&now_str())
```

**Also check:** `CatalogSiteDto` (`domain/catalog.rs`) — add `updated_at` field with `#[serde(default)]` so it survives deserialization even when API omits it.

**Verification:** Bootstrap/sync-pull sites family must succeed without NOT NULL errors.

**Acceptance:** `sites` family appears in `families_synced` list, NOT in `errors` list.

---

## 5. Fix D — Bootstrap Catalog Write Order

**Bug:** `catalog_items` fails with FK constraint because items reference category_id/unit_id that don't exist yet locally.

**Root cause:** Bootstrap writes: items → categories → units (bootstrap.rs lines 90-108). Items have FK to categories(id) and units(id). Categories/units must be written FIRST.

**File to change:**
- `crates/warehouse_core/src/sync/bootstrap.rs`, lines 89-108

**Fix approach:** Reorder writes: categories → units → items. Also add error propagation so a category/unit write failure prevents items write.

```rust
// Step 4a: Categories (no FK deps)
self.sync_catalog_categories().await
// Step 4b: Units (no FK deps)
self.sync_catalog_units().await
// Step 4c: Items (FK → categories, units)
self.sync_catalog_items().await
```

**Also fix:** `snapshot_writer.rs` `write_items()` — remove `PRAGMA foreign_keys = OFF/ON` since SQLite ignores PRAGMA changes mid-transaction. Instead, rely on correct write order.

**Verification:** Bootstrap on clean DB — items, categories, units all succeed without FK errors.

**Acceptance:** All three families in `families_synced`, zero FK errors.

---

## 6. Fix E — Operation & Asset DTO IDs: `i64` → `String`/`Uuid`

**Bug:** `operations list` fails with `invalid type: string <uuid>, expected i64`. SyncServer returns UUID strings for operation IDs.

**Affected types (all in `domain/`):**
- `operation.rs`: `OperationResponse.id` (line 134), `OperationListItem.id` (line 192), `OperationLineResponse.id` (line 115)
- `assets.rs`: `PendingAcceptanceRow.operation_id` (line 6), `LostAssetRow.operation_id` (line 23), `IssuedAssetRow.operation_id` (line 38)
- `operation.rs`: `OperationLineResponse.id` — line IDs also may be UUID
- `operation.rs`: `AcceptLineRequest.line_id` (line 170)

**Fix approach:** Change all operation/line IDs from `i64` to `String`. Add `#[serde(deserialize_with = "...")]` or use `#[serde(alias)]` to handle both string and numeric deserialization for backward compat.

**Files to change:**
- `crates/warehouse_core/src/domain/operation.rs`
- `crates/warehouse_core/src/domain/assets.rs`
- `crates/warehouse_core/src/storage/snapshot_writer.rs` (write_operations, write_pending_acceptance, write_lost_assets, write_issued_assets)
- `crates/warehouse_core/src/storage/repos.rs` (any repo queries referencing operation_id)

**SQLite schema consideration:** `operation_drafts.draft_id` is `TEXT PRIMARY KEY`. `pending_acceptance_balances.operation_id` is `INTEGER`. If operation IDs become UUID strings, the asset tables need schema migration to change INTEGER → TEXT. **Decision needed**: either (a) migration 0005 to change column types, or (b) store raw UUID strings and add index on operation_id text. **Recommendation (a)**: add migration 0005.

**Verification:**
```powershell
cargo run -p warehouse_cli -- operations list 1
cargo run -p warehouse_cli -- assets pending
cargo run -p warehouse_cli -- assets lost
```

**Acceptance:** Operations list returns parsed rows. Asset commands return parsed rows. No `invalid type` errors.

---

## 7. Fix F — Sync Device ID: `Uuid` → `i32`

**Bug:** SyncServer `/ping`, `/push`, `/pull`, `/bootstrap/sync` expect integer `device_id`. Core sync DTOs model it as `Uuid`.

**Affected types (`domain/sync_types.rs`):**
- `PingRequest.device_id` (line 10)
- `PushRequest.device_id` (line 56)
- `PullRequest.device_id` (line 87)
- `BootstrapResponse.device_id` (line 109)
- `PullEvent.source_device_id` (line 80)

**Also affected:**
- `ids.rs`: `CoreIds.device_id: Option<Uuid>` → `Option<i32>`
- `auth/profile.rs`: `Profile.device_id: uuid::Uuid` → `i32`
- `domain/auth.rs`: `AuthDeviceInfo.id: Uuid` → `i32`
- `sync/pull.rs` line 614: `device_id: uuid::Uuid::default()` → `0` or actual device_id

**Fix approach:** Change all sync-protocol and identity device_id fields from `Uuid` to `i32`. Add `#[serde(default)]` for robustness.

**Files to change:**
- `crates/warehouse_core/src/domain/sync_types.rs`
- `crates/warehouse_core/src/ids.rs`
- `crates/warehouse_core/src/auth/profile.rs`
- `crates/warehouse_core/src/domain/auth.rs`
- `crates/warehouse_core/src/sync/pull.rs`
- `crates/warehouse_core/src/sync/engine.rs` (any ping/push calls)
- `crates/warehouse_core/src/syncserver/device_sync.rs` (any request construction)

**Verification:** `sync-pull` device_events family must complete without deserialization errors.

**Acceptance:** No `device_id` integer parsing errors in error_log.

---

## 8. Fix G — `BalanceRow.site_code` Required but API Returns `site_name`

**Bug:** Balance endpoint deserialization fails because `BalanceRow.site_code` is non-optional but actual SyncServer response contains `site_name` (not `site_code`).

**File to change:**
- `crates/warehouse_core/src/domain/balance.rs`, line 7

**Fix approach:**
```rust
#[serde(alias = "site_name")]
pub site_code: String,
```
Or change field name to `site_name` and make `site_code` an alias.

**Verification:** `balances list 1` must succeed without deserialization errors.

**Acceptance:** Balances list returns parsed rows from real stand.

---

## 9. Fix H — Page Sizes Capped to Server Limits

**Bug:** Core uses page sizes (500, 200) that may exceed SyncServer endpoint limits. Smoke report found `page_size <= 100/200 validation` errors.

**Affected locations (`sync/pull.rs`):**
- `pull_catalog_items()` line 278: `Some(500)` → `Some(200)`
- `pull_catalog_categories()` line 302: `Some(500)` → `Some(200)`
- `pull_catalog_units()` line 325: `Some(500)` → `Some(200)`
- `pull_recipients()` line 346: `200u32` (already OK)
- `pull_balances()` line 375: `200u32` (already OK)
- `pull_pending_acceptance()` line 406: `200u32` (already OK)
- `pull_lost_assets()` line 435: `200u32` (already OK)
- `pull_issued_assets()` line 464: `200u32` (already OK)
- `pull_temporary_items()` line 495: `200u32` (already OK)
- `pull_operations()` line 525: `100u32` (already OK)
- `pull_stock_summary()` line 591: `500` → `200`
- `pull_device_events()` line 616: `Some(200)` (already OK)

**Also affected (`sync/bootstrap.rs`):**
- `sync_catalog_items()` line 181: `Some(500)` → `Some(200)`
- `sync_catalog_categories()` line 200: `Some(500)` → `Some(200)`
- `sync_catalog_units()` line 218: `Some(500)` → `Some(200)`

**Fix approach:** Cap all limits at 200. Define `const DEFAULT_PAGE_SIZE: u32 = 200;` in a shared location.

**Files to change:**
- `crates/warehouse_core/src/sync/pull.rs`
- `crates/warehouse_core/src/sync/bootstrap.rs`

**Verification:** No `page_size` validation errors in error_log after sync-pull.

**Acceptance:** All families pull without page-size rejection.

---

## 10. Fix I — `documents_list` Response Shape Mismatch

**Bug:** `documents_list(offset, limit)` expects `PaginatedResponse<DocumentDto>` but actual endpoint returns different shape (uses offset/limit, not page/page_size).

**File to change:**
- `crates/warehouse_core/src/syncserver/documents.rs`

**Fix approach:** Check actual SyncServer `/documents` endpoint response. If it returns `{ items: [...], total_count: N }` without `page`/`page_size` fields, create a separate `OffsetPaginatedResponse<T>` DTO or add `#[serde(default)]` to `page`/`page_size` in `PaginatedResponse`.

Simplest fix: add `#[serde(default)]` to `page` and `page_size` fields in `PaginatedResponse` (`domain/pagination.rs` lines 23-24).

**Verification:** `cargo run -p warehouse_cli -- sync-pull` must not fail on documents family with deserialization errors.

**Acceptance:** Documents family completes without pagination shape errors.

---

## 11. Fix J — Pull Failures Preserve Family Name

**Bug:** Pull errors are reported as `"unknown"` family in `SyncRunSummary` because `log_result()` hardcodes `name: "unknown"` on `Err(e)` path.

**File to change:**
- `crates/warehouse_core/src/sync/pull.rs`, lines 636-656

**Fix approach:** Change `log_result` signature to accept a family name parameter:

```rust
async fn log_result(&self, summary: &mut SyncRunSummary, family_name: &str, result: CoreResult<FamilyResult>) {
    match result {
        Ok(family) => { summary.families.push(family); ... }
        Err(e) => {
            summary.families.push(FamilyResult {
                name: family_name,
                success: false,
                items_count: 0,
                error: Some(e.to_string()),
            });
            summary.errors_count += 1;
        }
    }
}
```

Update all callers to pass the family name string (or infer from the `FamilyResult::name` if we restructure the error to carry name).

Alternative: change per-family pull methods to return `FamilyResult` with name set even on error, then `log_result` preserves it.

**Verification:** Inject a forced failure in a test. Verify `SyncRunSummary.families` contains the correct family name in the error entry.

**Acceptance:** CLI `sync-pull` output shows actual family names for failures, not `"unknown"`.

---

## 12. Fix K — Bootstrap/Sync Engine Failure Semantics

**Bug A (bootstrap):** `bootstrap.rs` line 131: success only checks health/protocol/identity failures. Catalog and site failures are printed in `errors` but don't affect `success`.

**Bug B (sync engine):** `engine.rs` line 235: `result.success = result.error.is_none()`. Pull phase does not set `result.error` on family failures. `pull.rs` `pull_all()` returns `SyncRunSummary` but the engine ignores family-level errors.

**Files to change:**
- `crates/warehouse_core/src/sync/bootstrap.rs`, line 131-133
- `crates/warehouse_core/src/sync/engine.rs`
- `crates/warehouse_core/src/sync/pull.rs`

**Fix A (bootstrap):** Change success determination to include required families. Identify which families are "required" (sites, catalog_items, catalog_categories, catalog_units) vs "optional" (documents, reports):

```rust
let required_families = ["catalog_items", "catalog_categories", "catalog_units", "sites"];
let has_required_failure = result.errors.iter().any(|e| {
    required_families.iter().any(|f| e.starts_with(f))
});
result.success = !result.errors.iter().any(|e| {
    e.starts_with("health") || e.starts_with("protocol") || e.starts_with("identity")
}) && !has_required_failure;
```

**Fix B (sync engine):** After `pull_all()` returns `SyncRunSummary`, propagate family-level errors to the engine's `SyncResult`. If any required family failed, set `result.error` and `result.success = false`.

**Fix C (CLI output):** `facade/mod.rs` / `main.rs` must show degraded/failure status clearly. `SyncStatusInfo` must include required family failure list.

**Verification:** Inject a forced catalog_items failure. Bootstrap must return `success: false`. Sync engine `full` must return failure status.

**Acceptance:** Bootstrap/sync-pull do NOT report plain `SUCCESS` when required data families fail.

---

## 13. Required Test Ladder

### Static Checks (after every fix)
```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-run
```

### Unit Tests (after fixes E, F, G)
- DTO serde roundtrips for operation UUID IDs
- DTO serde for balance site_name alias
- Device ID integer deserialization
- PaginatedResponse with missing page/page_size defaults

### Component Tests (after fixes A, B, C, D, K)
- SQLite creation on missing file path
- Draft save/load with correct operation_type
- Sites writer with updated_at
- Catalog bootstrap write order (categories → units → items)
- Bootstrap success/failure with injected family errors
- Pull family name preservation on error

### Integration Tests (after all fixes)
- Real SQLite migrated schema (including migration 0005 if added)
- Real SyncServer stand at `http://127.0.0.1:8000`

### Real Stand Smoke (final gate)
```powershell
# Clean profile
Remove-Item -Force "$env:LOCALAPPDATA\warehouse_client_core\warehouse.db" -ErrorAction SilentlyContinue

cargo run -p warehouse_cli -- health-remote
cargo run -p warehouse_cli -- db init
cargo run -p warehouse_cli -- auth-context
cargo run -p warehouse_cli -- site set 1
cargo run -p warehouse_cli -- bootstrap
cargo run -p warehouse_cli -- sync-pull
cargo run -p warehouse_cli -- catalog search test
cargo run -p warehouse_cli -- balances list 1
cargo run -p warehouse_cli -- operations list 1
cargo run -p warehouse_cli -- temp-items list
cargo run -p warehouse_cli -- assets pending
cargo run -p warehouse_cli -- assets lost
cargo run -p warehouse_cli -- assets issued
cargo run -p warehouse_cli -- draft create RECEIVE --site-id 1
cargo run -p warehouse_cli -- draft list
cargo run -p warehouse_cli -- outbox list
```

### UI Automation
Not applicable for Core (no UI). Left unchecked with note.

### User Scenarios
1. First launch: empty profile → db init → auth → site set → bootstrap (green)
2. Online pull: sync-pull all families green
3. Offline read: catalog search, balances, operations, assets from local cache
4. Draft lifecycle: create → list → get → validate → delete

### Regression
- auth context and site scopes
- catalog tree/search
- balances/assets
- operation UUID IDs
- sync cursor store
- FFI handle lifecycle

---

## 14. Real Test Stand

### Database
- SyncServer: PostgreSQL via SSH tunnel on `localhost:5434`
- Core: clean SQLite profile, deleted before each smoke run

### Seed Data
- Valid root user token (`SYNC_USER_TOKEN` / `WAREHOUSE_USER_TOKEN`)
- Valid device token (`SYNC_DEVICE_TOKEN` / `WAREHOUSE_DEVICE_TOKEN`)
- At least one active site (id=1)
- Catalog items/categories/units
- At least one operation (with UUID id)
- At least one balance row

### Services
- SyncServer API: `http://localhost:8000`, health at `GET /api/v1/health`
- Django: `http://localhost:8001`, health at `GET /healthz/`

### Stand Availability Protocol
1. Probe `curl http://localhost:8000/api/v1/health` and `curl http://localhost:8001/healthz/`
2. If stand is running → proceed
3. If stand is NOT running → STOP, report: **«Стенд не обнаружен. Подними стенд (Django :8001 + SyncServer :8000 + SSH-туннель :5434).»**
4. Agent does NOT start the stand itself. Waits for user confirmation.

### Environment Variables (names only)
- `SYNC_USER_TOKEN`
- `WAREHOUSE_USER_TOKEN`
- `SYNC_DEVICE_TOKEN`
- `WAREHOUSE_DEVICE_TOKEN`
- `SYNC_TEST_BASE_URL`

---

## 15. Files Changed Summary

| Fix | Files |
|-----|-------|
| A | `storage/mod.rs` |
| B | `storage/repos.rs` (lines 784-799) |
| C | `storage/snapshot_writer.rs` (lines 127-128), `domain/catalog.rs` |
| D | `sync/bootstrap.rs` (lines 89-108), `storage/snapshot_writer.rs` (write_items PRAGMA) |
| E | `domain/operation.rs`, `domain/assets.rs`, `storage/snapshot_writer.rs`, `storage/repos.rs`, `migrations/sqlite/0005_*.sql` (NEW) |
| F | `domain/sync_types.rs`, `ids.rs`, `auth/profile.rs`, `domain/auth.rs`, `sync/pull.rs`, `sync/engine.rs` |
| G | `domain/balance.rs` (line 7) |
| H | `sync/pull.rs`, `sync/bootstrap.rs` |
| I | `domain/pagination.rs`, `syncserver/documents.rs` |
| J | `sync/pull.rs` (log_result) |
| K | `sync/bootstrap.rs` (line 131), `sync/engine.rs`, `facade/mod.rs` |

---

## 16. Migration 0005 (If Added for Fix E)

If asset tables change `operation_id` from `INTEGER` to `TEXT`:

```sql
-- Migration 0005: UUID operation IDs in asset tables
ALTER TABLE pending_acceptance_balances RENAME TO _old_pending;
-- recreate with TEXT operation_id
-- copy data
-- drop _old_pending
```

**Decision point:** Executor evaluates whether a simpler approach (storing as TEXT without ALTER, relying on SQLite flexible typing) works. If not, full migration is needed.

---

## 17. Dependency Order

```
Fix A (SQLite creation)
  ↓
Fix B (draft binding) ── independent, can run parallel with A
  ↓
Fix C (sites updated_at)
  ↓
Fix D (catalog write order) ── depends on A+B for clean DB testing
  ↓
Fix E (operation UUID) ── affects many files; run after A-D pass
  ↓
Fix F (device id) ── independent of E
  ↓
Fix G (balance site_name) ── independent
  ↓
Fix H (page sizes) ── independent
  ↓
Fix I (documents shape) ── independent
  ↓
Fix J (family names) ── depends on H for testability
  ↓
Fix K (failure semantics) ── depends on J
  ↓
Stand smoke (all fixes together)
```

Fixes B, G, H, I are truly independent and can be parallelized if multiple executors are available.

---

## 18. Evidence Table Template

| Check | Command / Tool | Result | Evidence |
|---|---|---|---|
| Static checks | `cargo fmt`, `cargo clippy`, `cargo test --no-run` | pass/fail/skipped | output summary |
| Unit tests | `cargo test --workspace` | pass/fail/skipped | test count |
| Component tests | SQLite/mock tests | pass/fail/skipped | test names |
| DB integration | clean missing-file profile + migrations | pass/fail/skipped | DB path |
| Stand auth/bootstrap | CLI commands | pass/fail/skipped | no-secret output |
| Stand sync/read | CLI commands | pass/fail/skipped | families/items/errors |
| Draft/outbox | CLI commands | pass/fail/skipped | draft id (safe to show) |
| Regression | full pack | pass/fail/skipped | summary |

### 2026-05-20 stand evidence

- Health probes: SyncServer `200`, Django `200`.
- `cargo run -p warehouse_cli -- auth-context` → pass.
- `cargo run -p warehouse_cli -- site set 1` → pass.
- `cargo run -p warehouse_cli -- bootstrap` → fail:
  - synced: `catalog_categories`, `sites`
  - errors: `catalog_units: empty response`, `catalog_items: FOREIGN KEY constraint failed`
- `cargo run -p warehouse_cli -- sync-pull` → fail summary: `19 families, 5 items, 10 errors`.
  - fail families visible by name: `catalog_items`, `balances`, `lost_assets`, `temporary_items`, `operations`, `documents`, `stock_summary`
- Recent `error_log` evidence after stand run:
  - `FOREIGN KEY constraint failed`
  - `missing field \`unit_symbol\``
  - `missing field \`site_code\``
  - `missing field \`total_count\``
  - `missing field \`item_name\``
  - `page_size <= 100` validation error
- CLI smoke reads:
  - `catalog search bolt` → pass (0 items, command succeeds)
  - `balances list 1` → pass (empty local result, command succeeds)
  - `operations list 1` → fail: `missing field \`site_code\``
  - `temp-items list` → pass (empty local result)
  - `assets pending|lost|issued` → pass (empty local result)
- Draft smoke:
  - `draft create RECEIVE --site-id 1` → pass
  - `draft list` → pass

---

## 19. Final Acceptance Criteria

This TZ is complete only when ALL of the following are true:

1. Clean profile first-run works without manual DB file creation (`db init` on empty directory succeeds).
2. Bootstrap completes with all required families (health, protocol, identity, catalog_items, catalog_categories, catalog_units, sites) — no errors.
3. Sync-pull completes with required families — no deserialization or FK errors.
4. CLI smoke commands pass: `catalog search`, `balances list`, `operations list`, `temp-items list`, `assets pending|lost|issued`.
5. Draft create/list/get/validate works on clean SQLite.
6. Failed families report actual family names, not `"unknown"`.
7. Bootstrap/sync engine reports failure (not SUCCESS) when required families fail.
8. `docs/CORE_STAND_SMOKE_REPORT.md` is updated with passing evidence.
9. `docs/TZ_CORE_CLIENT_READY_COMPLETION.md` Levels 4-5 are unblocked.
10. `docs/TZ_CORE_STAND_CONTRACT_FIXES_BEFORE_AIWORKSTATION.md` checklist items updated to reflect completion.
