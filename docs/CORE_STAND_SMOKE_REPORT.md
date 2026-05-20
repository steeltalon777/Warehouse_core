# Core Stand Smoke Report

Date: 2026-05-19
Project: `Warehouse_client_core`
Stand: SyncServer `http://127.0.0.1:8000`, Django `http://localhost:8001`

## Purpose

Validate whether the Rust core is ready to become the foundation for the upcoming `WarehouseAIWorkstation` migration.

This report covers the required gate before WPF migration:

- static Rust checks;
- local SQLite/profile lifecycle;
- real SyncServer health/auth/site/bootstrap/sync smoke;
- read-only facade smoke for catalog/balances/operations/assets;
- blocker list for Core Levels 4-5.

No application source code was changed.

## Stand And Secret Handling

Tokens were loaded from local environment files into process environment variables only. Secret values are intentionally not recorded in this report.

Environment variable names used:

- `WAREHOUSE_USER_TOKEN`
- `SYNC_USER_TOKEN`
- `WAREHOUSE_DEVICE_TOKEN`
- `SYNC_DEVICE_TOKEN`

## High-Level Result

**Gate result: NOT PASSED.**

The Rust core compiles and local unit/component tests pass, and the core can authenticate against the real SyncServer. However, the real stand smoke uncovered contract and persistence mismatches that block using the core as the AIWorkstation migration foundation.

The following workflows are proven:

- remote health check;
- local health check;
- auth context loading with real user token;
- site list and active site selection;
- partial bootstrap;
- partial sync-pull;
- local catalog search after partial sync;
- empty balances/assets/temp-items list rendering.

The following workflows are blocked:

- clean first-run DB creation without pre-created SQLite file;
- successful full bootstrap without family errors;
- successful full `sync-pull` without family errors;
- operations list deserialization;
- draft creation persistence;
- device `/pull` event stream;
- full sync engine run without errors.

## Evidence Summary

| Check | Command / Tool | Result | Notes |
|---|---|---|---|
| SyncServer health | `GET /api/v1/health` | pass | HTTP 200 |
| Django health | `GET /healthz/` | pass | HTTP 200 |
| Rust format | `cargo fmt --all -- --check` | pass | no output |
| Rust clippy | `cargo clippy --workspace --all-targets -- -D warnings` | pass | clean with `CARGO_INCREMENTAL=0` |
| Rust tests | `cargo test --workspace` | pass | 40/40 tests pass |
| FFI release build | `cargo build -p warehouse_ffi --release` | pass | `warehouse_ffi.dll` built, ~9.8 MB |
| Local health | `cargo run -p warehouse_cli -- health-local` | pass | `Status: OK` |
| Remote health | `cargo run -p warehouse_cli -- health-remote` | pass | `Remote health: OK` |
| Auth context | `cargo run -p warehouse_cli -- auth-context` | pass | root profile and sites returned |
| Site set | `cargo run -p warehouse_cli -- site set 1` | pass | active site set to `1` |
| Bootstrap | `cargo run -p warehouse_cli -- bootstrap` | partial/fail | reports `SUCCESS` but has family errors |
| Sync pull | `cargo run -p warehouse_cli -- sync-pull` | partial/fail | 19 families, 23 items, 13 errors |
| Catalog search | `cargo run -p warehouse_cli -- catalog search test` | pass after partial sync | 5 items returned |
| Operations list | `cargo run -p warehouse_cli -- operations list 1` | fail | operation id DTO mismatch |
| Draft create | `cargo run -p warehouse_cli -- draft create RECEIVE --site-id 1` | fail | local DB constraint mismatch |

## Smoke Steps Performed

### 1. Static and local tests

Commands executed:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build -p warehouse_ffi --release
```

Result:

- format check passed;
- clippy passed;
- unit/component tests passed: 40 tests;
- FFI release DLL built.

### 2. Stand availability

Commands executed:

```powershell
curl http://localhost:8000/api/v1/health
curl http://localhost:8001/healthz/
```

Result: both returned HTTP 200.

### 3. Clean profile setup

A clean smoke profile was attempted at:

```text
C:\Users\steel\AppData\Local\warehouse_client_core\warehouse.db
```

The previous profile was backed up before the test and restored after the test. The smoke DB was preserved as:

```text
C:\Users\steel\AppData\Local\warehouse_client_core\warehouse.db.smoke-20260519-2135
```

Finding:

- `CoreHandle::open` / CLI `db init` failed when the SQLite file did not exist.
- After manually creating an empty file, `db init` succeeded and migrations `[1, 2, 3, 4]` applied.

This is a first-run blocker for desktop/mobile clients because bootstrap cannot require a pre-created DB file.

### 4. Auth/site smoke

`auth-context` returned root profile metadata and available sites. `site set 1` succeeded.

This proves token headers and `/auth/context` integration work at least for the root user on the current stand.

## Bootstrap Result

Command:

```powershell
cargo run -p warehouse_cli -- bootstrap
```

Observed output:

```text
Bootstrap: SUCCESS
Protocol: ok
Families synced: ["catalog_categories", "catalog_units"]
Errors:
  - catalog_items: Database error: FOREIGN KEY constraint failed
  - sites: Database error: NOT NULL constraint failed: sites.updated_at
```

Assessment:

- The CLI reports `SUCCESS` even though family errors are present.
- `catalog_categories` and `catalog_units` were written.
- `catalog_items` failed due to local FK ordering/constraint handling.
- `sites` failed because local schema requires `sites.updated_at`, but writer inserts only `site_id`, `code`, `name`, `is_active`.

This means bootstrap is not safe as a first-run gate yet.

## Sync Pull Result

Command:

```powershell
cargo run -p warehouse_cli -- sync-pull
```

Observed output:

```text
Sync pull: 19 families, 23 items, 13 errors
OK health (1 items)
OK auth (0 items)
OK catalog_items (22 items)
OK recipients (0 items)
OK pending_acceptance (0 items)
OK issued_assets (0 items)
13 FAIL unknown rows
```

Notes:

- Failed family names are reported as `unknown`, making the CLI summary hard to diagnose.
- Error details were found in local `error_log`.
- Some families can sync; full sync does not yet meet Level 5 acceptance.

## Error Log Findings

Distinct error classes from the smoke DB `error_log`:

| Error class | Count | Interpretation |
|---|---:|---|
| `sites.updated_at` NOT NULL violation | 3 | Core writer/schema mismatch for sites |
| FK constraint in `catalog_items` | 2 | Core item write order/constraint handling issue |
| UUID operation id expected as `i64` | 3 | Core DTO expects integer operation IDs; SyncServer returns UUID strings |
| UUID asset operation id expected as `i64` | 3 | Same mismatch in asset/lost rows |
| missing `site_code` | 12 total across variants | Core DTO expects `site_code`; SyncServer rows expose `site_name` but not `site_code` in several responses |
| missing `total_count` | 3 | Core expects `PaginatedResponse`; at least one endpoint response is not that shape |
| `page_size <= 100/200` validation | 6 total | Core uses page sizes above current endpoint limits |
| `device_id` integer parsing | 3 | Core uses UUID for `/pull`/sync DTOs; SyncServer schema expects integer `device_id` |

These are contract/DTO/persistence mismatches, not transient stand failures.

## API Shape Samples

Sampled without recording token values:

- `/api/v1/catalog/sites` returns `sites[]` with `site_id`, `code`, `name`, `is_active`, `permissions`, `server_time`; no `updated_at`.
- `/api/v1/operations` returns operation `id` as UUID string.
- `/api/v1/balances/by-site` returns `site_id`, `site_name`, `inventory_subject_id`, item fields, `qty`, `updated_at`; no `site_code`.

Additional sampled endpoint shapes:

- `/api/v1/lost-assets` returns `operation_id` as UUID string, not integer.
- `/api/v1/reports/stock-summary` accepts smaller `page_size` values and returns paginated data with `total_count`.
- `/api/v1/documents?offset=0&limit=2` returns `items[]`; Core currently expects a generic paginated response for documents.
- `/api/v1/pull` rejects UUID `device_id`; SyncServer schema expects integer `device_id`.

## Read Commands

Commands and results:

```powershell
cargo run -p warehouse_cli -- catalog search test
cargo run -p warehouse_cli -- balances list 1
cargo run -p warehouse_cli -- operations list 1
cargo run -p warehouse_cli -- temp-items list
cargo run -p warehouse_cli -- assets pending
cargo run -p warehouse_cli -- assets lost
cargo run -p warehouse_cli -- assets issued
```

Results:

- catalog search returned 5 items;
- balances/temp/assets commands rendered empty lists successfully;
- operations list failed with `invalid type: string <uuid>, expected i64`.

## Draft/Outbox Smoke

Commands:

```powershell
cargo run -p warehouse_cli -- draft create RECEIVE --site-id 1
cargo run -p warehouse_cli -- draft list
cargo run -p warehouse_cli -- outbox list
```

Result:

```text
Error: Database error: NOT NULL constraint failed: operation_drafts.operation_type
No drafts.
No outbox events.
```

Assessment:

- Draft creation path is not yet usable against the migrated local schema.
- Outbox cannot be tested until draft creation or another queueing path works.

## Sync Engine Smoke

Commands:

```powershell
cargo run -p warehouse_cli -- sync pull
cargo run -p warehouse_cli -- sync full
```

Result: both commands reported `SUCCESS`, but included pull errors.

Observed sync engine output:

```text
Sync mode: Some(PullOnly) — SUCCESS
  Push: 0 accepted, 0 failed, 0 conflicts
  Pull: 1 items, 11 errors
  Bootstrap: N/A

Sync mode: Some(Full) — SUCCESS
  Push: 0 accepted, 0 failed, 0 conflicts
  Pull: 1 items, 11 errors
  Bootstrap: OK
```

Assessment:

- The engine success flag does not currently fail on family-level pull errors.
- This is unsafe for WPF first-run/bootstrap UX: the app could proceed with incomplete cache.

## Required Fixes Before AIWorkstation Migration

1. Fix SQLite first-run creation so `CoreHandle::open` and CLI `db init` create a missing DB file reliably on Windows.
2. Fix `sites` schema/writer mismatch (`updated_at` required but not inserted).
3. Fix catalog bootstrap order or FK strategy so items can be written after categories/units/subjects are available.
4. Align operation IDs in Core DTOs with SyncServer UUID operation IDs.
5. Align asset/document/report DTOs with real SyncServer payloads.

6. Align `/ping`, `/push`, `/pull`, `/bootstrap/sync` device identity types: SyncServer expects integer `device_id`; Core currently models UUID in several sync DTOs.
7. Lower Core page sizes to endpoint limits or read limits from contract metadata.
8. Preserve real family names on pull failure instead of reporting `unknown`.
9. Make bootstrap/sync engine return failure when required families fail.
10. Fix draft persistence mismatch (`operation_drafts.operation_type` NOT NULL failure).

## Gate Decision

Core Levels 4-5 are **not complete**.

Do not start `WarehouseAIWorkstation` migration Layer 0 (`C# FFI wrapper`) until at least the following smoke passes on a clean profile:

```text
health-remote
init/open clean SQLite profile
auth-context
site set
bootstrap without errors
sync-pull without required-family errors
catalog search from local cache
operations list
balances list
sync pull/full report failure correctly when any required family fails
```

## Recommended Next TZ

Create a short Core stabilization TZ before the AIWorkstation migration TZ:

`TZ: Warehouse_client_core Stand Contract Fixes Before WPF Migration`

Scope:

- fix DTO contract mismatches found here;
- fix SQLite writer/schema mismatches;
- add stand smoke commands as repeatable tests;
- update `TZ_CORE_CLIENT_READY_COMPLETION.md` evidence only after smoke passes.
