# TZ: Warehouse_client_core Stand Contract Fixes Before AIWorkstation Migration

## Execution Checklist

- [ ] 0. Context verified
- [ ] 1. Architecture boundaries confirmed
- [ ] 2. Implementation level 1 complete
- [ ] 3. Unit/component tests complete
- [ ] 4. Integration tests with real dependencies complete
- [ ] 5. Stand smoke tests complete
- [ ] 6. UI automation tests complete
- [ ] 7. User scenario tests complete
- [ ] 8. Regression checks complete
- [ ] 9. Documentation updated
- [ ] 10. Final acceptance review complete

## Check Rules

- Architect creates this checklist and acceptance criteria.
- Executor agents may check implementation and test items only after running the required verification.
- QA verifier may check final acceptance only after reviewing evidence.
- Failed or unavailable checks stay unchecked with a blocker note.
- Do not start `WarehouseAIWorkstation` migration until this TZ passes Levels 0-5 at minimum.

## 0. Purpose

Close the real-stand blockers found in `docs/CORE_STAND_SMOKE_REPORT.md` so `Warehouse_client_core` can safely become the domain/offline foundation for `WarehouseAIWorkstation`.

## 1. Current Blockers

Real stand smoke on 2026-05-19 proved that static/unit checks pass, but full bootstrap/sync do not.

Blocking findings:

1. Clean SQLite first-run fails unless the DB file already exists.
2. `sites.updated_at` is `NOT NULL`, but site writer does not insert it.
3. Catalog item bootstrap can fail with FK constraint errors.
4. Core sync DTOs model `device_id` as UUID, while SyncServer `/ping`, `/push`, `/pull`, `/bootstrap/sync` schemas expect integer `device_id`.
5. Core operation DTOs expect integer operation IDs, while SyncServer returns UUID strings.
6. Several Core DTOs require `site_code`, while actual responses provide `site_id` and `site_name` only.
7. Core uses page sizes above endpoint limits.
8. Documents endpoint shape differs from Core's expected `PaginatedResponse`.
9. Pull family failures are reported as `unknown`.
10. Bootstrap/sync engine can report `SUCCESS` while required families failed.
11. Draft creation fails with `operation_drafts.operation_type` `NOT NULL` violation.

## 2. Architecture Boundaries

- SyncServer remains the source of truth for domain rules, balances, documents, reports, operation effects, and permissions.
- Rust core owns local SQLite profile, sync/outbox, DTO mapping, conflict state, and stable facade/FFI.
- WPF/AIWorkstation must not call SyncServer directly for warehouse domain flows after migration.

## 3. In Scope

- `crates/warehouse_core/src/config.rs`
- `crates/warehouse_core/src/storage/`
- `crates/warehouse_core/src/domain/`
- `crates/warehouse_core/src/syncserver/`
- `crates/warehouse_core/src/sync/`
- `crates/warehouse_core/src/operations/`
- `crates/warehouse_core/src/facade/`
- `crates/warehouse_cli/src/main.rs`
- FFI only when method signatures or error codes must reflect fixed facade contracts.
- Docs: `CORE_FACADE_V1.md`, `WPF_BINDINGS.md`, `SYNC_STAND.md`, and active TZ/report files.

## 4. Out Of Scope

- `WarehouseAIWorkstation` runtime code migration.
- `WarehouseDesktop` runtime code.
- SyncServer business-rule changes unless a contract gap is proven impossible to solve client-side.
- Direct database access by client runtime.
- AI module extraction or WPF UI redesign.

## 5. Level 0 — Context Verification

Executor must re-read:

- root `Functional and WorkLogik.md` sections for roles, operation types, lifecycle rules;
- `docs/CORE_STAND_SMOKE_REPORT.md`;
- `docs/TZ_CORE_CLIENT_READY_COMPLETION.md`;
- SyncServer schemas for sync, operations, balances, assets, documents, reports.

### Level 0 Acceptance

- Executor report lists inspected SyncServer schema files and Core DTO files.
- No code changes start before contract mismatches are mapped.
- Secret values are never printed or committed.

## 6. Level 1 — Contract Audit And DTO Alignment Plan

Create a table mapping actual SyncServer payloads to Core DTOs for these groups:

- auth/context and sites;
- `/catalog/sites`, `/catalog/items`, `/catalog/categories`, `/catalog/units`;
- `/balances`, `/balances/by-site`, `/balances/summary`;
- `/operations`, `/operations/{id}`;
- `/pending-acceptance`, `/lost-assets`, `/issued-assets`;
- `/temporary-items`;
- `/documents` and document render endpoints;
- `/reports/stock-summary`, `/reports/item-movement`;
- `/ping`, `/push`, `/pull`, `/bootstrap/sync`.

### Level 1 Acceptance

- Every smoke error from `CORE_STAND_SMOKE_REPORT.md` is mapped to a concrete DTO/schema/writer mismatch.
- Any required SyncServer contract change is documented separately as blocker, not silently patched in Core.
- Core DTO types use SyncServer source-of-truth types unless an ADR explicitly says otherwise.

## 7. Level 2 — SQLite First-Run And Snapshot Writer Fixes

Fix local profile persistence issues found by stand smoke.

Required changes:

1. Missing SQLite file must be created/opened by `CoreHandle::open` and CLI `db init` without manual file pre-creation.
2. Site writer must satisfy local schema (`updated_at`, `created_at` if required).
3. Catalog bootstrap must write prerequisites before dependent records or use a safe transaction strategy.
4. FK handling must be deterministic; do not rely on `PRAGMA foreign_keys` toggles inside a transaction if SQLite ignores them there.
5. Snapshot writes must leave local cache in a consistent state when one family fails.

### Level 2 Acceptance

- Empty profile directory → `warehouse_cli db init` succeeds.
- Empty profile directory → `auth-context`, `site set`, and `bootstrap` can create/open DB without manual file creation.
- Bootstrap writes sites, units, categories, and items without FK/NOT NULL errors.
- Component tests cover empty-file and missing-file SQLite creation.

## 8. Level 3 — SyncServer DTO Contract Fixes

Align Core DTOs, HTTP clients, and local writers with real SyncServer responses.

Required fixes:

1. Operation IDs must be UUID strings/UUID values, not `i64`, wherever SyncServer returns UUID.
2. Asset rows containing `operation_id` must accept UUID operation IDs.
3. Device sync DTOs must match SyncServer integer `device_id` contract or document/implement a server extension before using UUID.
4. DTOs must not require `site_code` where SyncServer responses only provide `site_id`/`site_name`.
5. Documents list parser must match actual `/documents` response shape.
6. Report and list clients must respect endpoint page-size limits.
7. Decimal/string quantity fields must parse consistently across balances, assets, operations, reports.

### Level 3 Acceptance

- Real sampled payloads from the stand deserialize in Core tests.
- DTO serde tests include operations UUID IDs, lost assets UUID operation IDs, missing `site_code`, documents list, and string decimals.
- No direct SyncServer response shape is guessed without a test fixture or live contract sample.

## 9. Level 4 — Pull/Bootstrap/Sync Engine Semantics

Fix orchestration behavior so UI clients can trust status.

Required behavior:

1. Failed pull families must preserve their actual family name, not `unknown`.
2. Bootstrap must return failure when required families fail.
3. Sync engine must return failure or degraded status when required pull/bootstrap families fail.
4. Optional families may be marked skipped/degraded only if documented and visible in `SyncRunSummary`.
5. `get_sync_status` must expose enough data for WPF: authenticated, active site, last success, last error, outbox count, required family failures.
6. CLI output must be suitable as smoke evidence.

### Level 4 Acceptance

- Injected/fake family failure test proves family names and failure status are preserved.
- Bootstrap cannot print plain `SUCCESS` when required data families fail.
- Sync `pull` and `full` produce machine-readable failure/degraded status.
- `list_sync_runs` is implemented or explicitly deferred with a documented reason.

## 10. Level 5 — Draft/Outbox Minimal Usability

Fix the local operation draft and outbox path enough to prove the offline intent foundation.

Required behavior:

1. `draft create RECEIVE --site-id <id>` creates a persisted local draft.
2. Draft operation type is stored using the local schema expected representation.
3. `draft list` and `draft get` show the created draft after process restart.
4. `draft validate` produces deterministic validation results.
5. `draft submit` queues outbox only for a valid draft.
6. `outbox list` shows queued events without exposing secrets.

### Level 5 Acceptance

- Draft create/list/get/validate works on a clean profile.
- Invalid draft validation errors are user-facing and stable.
- Outbox queueing path is covered by SQLite component test.
- Real stand push may remain a later gate only if explicitly documented; otherwise `outbox send` must be smoked.

## 11. Required Test Ladder

### Static checks

Required after every code change:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-run
```

### Unit tests

Required coverage:

- DTO serde for all changed endpoint groups;
- ID type conversions and UUID/int boundaries;
- page-size selection;
- sync family status aggregation;
- draft validation and persistence representation.

### Component tests

Required coverage:

- migrated SQLite schema on a missing DB file path;
- snapshot writer for sites/categories/units/items;
- pull service with fake clients for success and partial failure;
- draft/outbox repositories against real SQLite temp DB.

### Integration tests with real dependencies

Required coverage:

- real SQLite migrated schema;
- real SyncServer stand at `http://127.0.0.1:8000` or configured equivalent;
- real profile directory lifecycle;
- auth/context, sites, bootstrap, sync-pull, catalog, balances, operations, assets, documents/reports as applicable.

### Real stand smoke tests

Required smoke commands:

```powershell
cargo run -p warehouse_cli -- health-remote
cargo run -p warehouse_cli -- db init --path <clean-profile-db>
cargo run -p warehouse_cli -- auth-context
cargo run -p warehouse_cli -- site set <site-id>
cargo run -p warehouse_cli -- bootstrap
cargo run -p warehouse_cli -- sync-pull
cargo run -p warehouse_cli -- catalog search test
cargo run -p warehouse_cli -- balances list <site-id>
cargo run -p warehouse_cli -- operations list <site-id>
cargo run -p warehouse_cli -- temp-items list
cargo run -p warehouse_cli -- assets pending
cargo run -p warehouse_cli -- assets lost
cargo run -p warehouse_cli -- assets issued
```

### UI automation applicability

Core has no UI. FlaUI/Playwright are not applicable inside this TZ.

This checklist item remains unchecked unless a C# FFI smoke harness is added and QA accepts it as binding-level UI/client evidence.

### User scenario tests

Required Core scenarios:

1. First launch: missing DB/profile → bind tokens → auth context → set active site → bootstrap.
2. Offline read: pull online → stop stand or clear tokens → search catalog locally.
3. Sync status: failed family is visible and blocks success status.
4. Operations read: list operations from real stand and cache/read where supported.
5. Draft: create draft, restart process, list/validate draft.
6. Outbox: queue valid draft or document why outbox send is deferred.

### Regression pack

Must include:

- auth and role/site scopes;
- catalog tree/search;
- balances/assets rows;
- operations UUID IDs;
- document list/render DTOs;
- device sync DTOs;
- secret redaction;
- FFI handle lifecycle if FFI exports change.

## 12. Real Test Stand

### Database type and lifecycle

- SyncServer uses PostgreSQL through the user's maintained tunnel on `localhost:5434`.
- Agents must not start the SSH tunnel.
- Use only safe stand data; do not mutate production-like data without explicit user permission.
- Core profile DB must be a clean local SQLite file per smoke run.

### Seed data required

- at least one valid user token;
- valid device token;
- at least one active site with access;
- catalog categories, units, and items;
- balances for at least one site where available;
- at least one operation row with UUID id;
- optional temp items/assets/documents/reports for broader smoke.

### Services to probe

- SyncServer: `GET /api/v1/health`;
- Django: `GET /healthz/` when WPF migration planning depends on Django stand context.

### Environment variable names only

- `WAREHOUSE_USER_TOKEN`
- `SYNC_USER_TOKEN`
- `WAREHOUSE_DEVICE_TOKEN`
- `SYNC_DEVICE_TOKEN`
- `SYNC_TEST_BASE_URL`

## 13. Evidence Table Template

Executor completion report must include:

| Check | Command / Tool | Result | Evidence |
|---|---|---|---|
| Static checks | `cargo fmt`, `cargo clippy`, `cargo test --no-run` | pass/fail/skipped | summary/log path |
| Unit tests | `cargo test --workspace` | pass/fail/skipped | test count |
| SQLite integration | clean missing-file profile + migrations | pass/fail/skipped | DB path note |
| DTO contract | serde fixtures/live samples | pass/fail/skipped | endpoint list |
| Stand auth/bootstrap | CLI commands | pass/fail/skipped | no-secret output summary |
| Stand sync/read | CLI commands | pass/fail/skipped | families/items/errors |
| Draft/outbox | CLI commands | pass/fail/skipped | draft/outbox ids redacted if needed |
| Regression | pack list | pass/fail/skipped | summary |

## 14. Final Acceptance Criteria

This TZ is complete only when:

- clean profile first-run works without manual DB file creation;
- bootstrap completes without required-family errors;
- sync-pull completes without required-family errors or reports degraded/fail correctly;
- catalog, balances, operations, temp-items, and assets smoke commands do not fail on DTO/schema errors;
- draft create/list/validate works on clean SQLite;
- `docs/CORE_STAND_SMOKE_REPORT.md` is updated with passing evidence;
- `docs/TZ_CORE_CLIENT_READY_COMPLETION.md` Levels 4-5 may then be checked by executor/QA according to workflow rules.
