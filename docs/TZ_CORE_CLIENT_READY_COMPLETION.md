# TZ: Warehouse_client_core Client-Ready Completion Roadmap

## Execution Checklist

- [x] 0. Context verified
- [x] 1. Architecture boundaries confirmed
- [x] 2. Implementation level 1 complete
- [x] 3. Unit/component tests complete (Level 6: FFI error envelope + handle lifecycle; core: 8/8 pass)
- [ ] 4. Integration tests with real dependencies complete
- [ ] 5. Stand smoke tests complete
- [ ] 6. UI automation tests complete
- [ ] 7. User scenario tests complete
- [ ] 8. Regression checks complete
- [ ] 9. Documentation updated
- [ ] 10. Final acceptance review complete

## Check Rules

- Architect creates this checklist, levels, boundaries, and acceptance criteria.
- Executor agents may check implementation/test items only after implementation and verification evidence are attached in this file or in the executor report.
- QA verifier may check final acceptance only after reviewing all required evidence.
- Failed or unavailable checks stay unchecked with a blocker note; unit tests must not replace real stand checks.
- Follow `../../docs/AGENT_TZ_WORKFLOW.md` for the canonical TZ workflow.

## 0. Purpose

This TZ is the step-by-step assignment for bringing `Warehouse_client_core` from the current database/DTO foundation to a **client-ready Rust engine** that Android and WPF clients can build on.

It complements `docs/RUST_CORE_CAPABILITY_PLAN.md`, which is the broad capability map. This file is the executable roadmap for the missing client blockers: HTTP client, auth/bootstrap, pull sync, offline drafts, outbox/push, sync engine, stable facade, and FFI.

## 1. Current Status Snapshot

### 1.1 Already present

- Rust workspace: `Warehouse_client_core/Cargo.toml`.
- Crates: `crates/warehouse_core`, `crates/warehouse_ffi`, `crates/warehouse_cli`.
- Domain DTOs under `crates/warehouse_core/src/domain/`.
- `CoreConfig`, `CoreError`, `CoreHandle` skeleton.
- SQLite migration runner and initial/full schema migrations.
- Repository foundation in `storage/repos.rs`.
- CLI foundation with local health/config/database commands.
- Existing test foundation, currently reported as 8 tests.

### 1.2 Critical gaps before real clients can use the core

| Gap | Required result | Current state |
|---|---|---|
| HTTP client | Typed `SyncServerClient` for all required `/api/v1` endpoint groups using `reqwest` | Missing |
| Auth/bootstrap | Token binding, identity refresh, device registration/bootstrap, active site persistence | Missing |
| Pull sync | `PullSyncService` loads catalog, balances, operations, recipients, assets, temp items, document metadata | Missing |
| Offline operations | `OperationDraftService` creates/edits/validates drafts offline | Missing |
| Outbox + push | Queues commands, sends to SyncServer, retries, handles idempotency/conflicts | Missing |
| Sync engine | Orchestrates ping/push/pull/bootstrap with locking, progress, cancellation | Missing |
| Core facade | Stable UI-facing API for Kotlin/C#/CLI; no raw repo/HTTP access by clients | Stub only |
| FFI | Kotlin Android bindings and C# WPF bindings | Stub only |

## 2. Architecture Boundaries

### 2.1 Source of truth

- `SyncServer` is the authoritative source for warehouse domain data, access rules, operation effects, balances, documents, reports, temporary item resolution, and final validation.
- `Warehouse_client_core` caches server data and stores local user intent, but it must not become a second backend.
- Local SQLite is a working set, outbox, sync cursor store, and diagnostics cache.

### 2.2 Ownership split

| Owner | Owns |
|---|---|
| SyncServer | Final business rules, writes, access policies, balances, operation workflow, document/report generation |
| Rust core | SQLite profile, DTO mapping, SyncServer HTTP client, pull/push sync, outbox, draft validation, conflict state, stable facade, FFI-safe types |
| Android/WPF clients | UI, navigation, platform secure storage, camera/scanner, background scheduling, OS permissions, notifications |

### 2.3 Forbidden implementation paths

- Clients must not call SyncServer directly once the real core facade exists.
- Clients must not create parallel Room/SQLite/Entity Framework warehouse-domain stores.
- Core must not connect directly to the SyncServer database.
- Core must not log or hardcode user/device tokens.
- Core must not include Android/WPF UI logic.
- Core must not locally apply final balance effects as truth; previews must be marked as previews.

## 3. Client-Ready Definition

### 3.1 Read-only integration gate

Clients may start replacing surrogates for read-only screens when all are true:

- Level 0 through Level 6 are complete.
- FFI exposes open profile, bind token, refresh identity, set active site, pull sync, catalog search, balances, operation history, and sync status.
- A real SyncServer stand smoke test proves bootstrap/pull/offline read from SQLite.

### 3.2 Offline operation integration gate

Clients may implement real core-backed operation creation when all are true:

- Level 0 through Level 10 are complete.
- Operation drafts, outbox, push/retry, and sync engine are available through facade and FFI.
- A real stand scenario proves: offline draft -> queued submit -> reconnect -> server accepted/rejected -> pull refresh.

### 3.3 Production client-ready gate

Clients may treat the core as production-ready only when all are true:

- Level 0 through Level 12 are complete.
- Static, unit, component, integration, real stand, FFI smoke, user scenario, and regression checks pass.
- QA verifier checks final acceptance after evidence review.

## 4. Implementation Level Overview

| Level | Name | Gate produced |
|---|---|---|
| 0 | Contract freeze and ADRs | Safe start for production implementation |
| 1 | SyncServer HTTP client | Typed online access to required API groups |
| 2 | Auth, device bootstrap, profile context | Bound identity/site/device context |
| 3 | SQLite repositories and migrations complete | Durable local working set foundation |
| 4 | Pull sync and staged bootstrap | Offline read cache from real server |
| 5 | Read-only CoreFacade and CLI smoke | UI-facing read API without FFI |
| 6 | FFI foundation for read-only facade | Android/WPF can begin read-only integration | ✅ |
| 7 | Operation draft service | Offline draft creation/editing/validation |
| 8 | Outbox and push transport | Offline intent can reach SyncServer |
| 9 | Sync engine orchestration | Reliable background sync behavior |
| 10 | Full client facade through FFI | Clients can build real offline workflows |
| 11 | Contract, performance, chaos, regression hardening | Confidence under realistic load/failures |
| 12 | Documentation, packaging, client handoff | Production-ready handoff package |

## 5. Level 0 — Contract Freeze and ADRs

### 5.1 Scope

No broad production code before this level is complete except small diagnostics needed to inspect contracts.

### 5.2 Required decisions

Create or update ADRs under `../../docs/adr/`:

1. Core HTTP/sync contract source:
   - canonical source is SyncServer OpenAPI plus `../../API_MAP.md`;
   - DTO drift rules;
   - endpoint coverage matrix.
2. Outbox push strategy:
   - whether offline operation commands are pushed through `/push` device events;
   - or replayed through normal user-token operation endpoints by an outbox transport;
   - or split by command type.
3. FFI strategy:
   - Kotlin Android binding path, expected to be UniFFI unless rejected by proof;
   - C# WPF binding path, expected to be C ABI/PInvoke or chosen generator;
   - async runtime, cancellation, error envelopes, and memory ownership.
4. Token ownership:
   - platform secure storage owns secret token persistence;
   - Rust core receives tokens through a provider/callback or explicit bind method;
   - CLI/test profile may use environment variables only.

### 5.3 Deliverables

- `Warehouse_client_core/docs/contracts/CLIENT_READY_API_MATRIX.md`.
- `Warehouse_client_core/docs/contracts/CORE_FACADE_V1_DRAFT.md`.
- ADRs for HTTP/sync, outbox transport, FFI, and token ownership.
- Updated open questions list with any required SyncServer extensions.

### 5.4 Acceptance

- Every API group from `../../API_MAP.md` is classified as: required for client-ready, admin-only, compatibility-only, or out-of-scope.
- The team knows exactly which transport will send offline operation commands.
- No endpoint requires clients to bypass the core.

### 5.5 Verification

- Static documentation review by architect.
- QA verifies the matrix includes auth, catalog, operations, balances, temporary items, documents, recipients, assets, reports, device sync, health, and admin feature-gated endpoints.

## 6. Level 1 — SyncServer HTTP Client

### 6.1 Scope

Implement typed HTTP access in `crates/warehouse_core/src/syncserver/` or an equivalent clearly named module.

### 6.2 Required modules

- `client.rs`: base `SyncServerClient`, base URL normalization, `reqwest::Client`, timeout/retry policy.
- `auth.rs`: `/auth/me`, `/auth/sites`, `/auth/context`.
- `catalog.rs`: primary read, browse, category tree, admin feature-gated catalog writes.
- `operations.rs`: list/detail/create/update/effective-at/submit/cancel/accept-lines.
- `balances.rs`: `/balances`, `/balances/by-site`, `/balances/summary`.
- `temporary_items.rs`: list/detail/operations/approve-as-item/merge/delete.
- `documents.rs`: generate/read/render/list/status/operation document links.
- `recipients.rs`: list/detail/create/update/delete/merge.
- `assets.rs`: pending acceptance, lost assets, lost resolve, issued assets.
- `reports.rs`: item movement, stock summary.
- `device_sync.rs`: `/ping`, `/push`, `/pull`, `/bootstrap/sync`.
- `health.rs`: `/health`, `/ready`, detailed readiness/liveness.
- `admin.rs`: admin endpoints behind compile/runtime feature gate.

### 6.3 Required behavior

- Send `X-User-Token` (user flows) and `X-Device-Token` (device sync) where required.
- Never log token values.
- Map HTTP status and SyncServer payloads into stable `CoreError` categories: unauthenticated, forbidden, validation, conflict, timeout, unavailable, protocol mismatch, serialization.
- Preserve correlation/request IDs in diagnostics if SyncServer exposes them.
- Support paginated endpoints with explicit `Page<T>` DTOs.
- Add DTO version comments for every mapped endpoint group.

### 6.4 Acceptance

- `SyncServerClient` can call health, auth context, catalog read, operations list, balances list, and device ping against a real stand.
- Mock/component tests prove headers are correct and secrets are redacted.
- DTO serde tests cover representative success and error payloads from each required group.

### 6.5 Verification

- Static: `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`.
- Unit: URL/header construction, error mapping, serde roundtrips.
- Component: HTTP mock server for each subclient.
- Integration: real SyncServer stand smoke for health/auth/catalog/balances/operations/ping.

### 6.6 Level 1 Completion Evidence

- Implemented 13 subclient modules in `crates/warehouse_core/src/syncserver/`:
  - `client.rs`: `SyncServerClient` with `reqwest::Client`, token management (set/clear X-User-Token/X-Device-Token), `get`/`post`/`patch`/`delete` request builders with `AuthKind` enum, `send<T>` and `send_no_body` response helpers, HTTP→`CoreError` mapping (401→Auth, 403→Auth, 404→NotFound, 409→Conflict, 422→Validation, 5xx→Network).
  - `auth.rs`: `/auth/me`, `/auth/sites`, `/auth/context`, `/auth/sync-user`.
  - `catalog.rs`: cursor-synced `/catalog/items`, `/catalog/categories`, `/catalog/units` (updated_after/limit), `/catalog/categories/tree`, `/catalog/sites`.
  - `operations.rs`: list (paginated, filterable), get, create, update, set-effective-at, submit, cancel, accept-lines.
  - `temporary_items.rs`: list, get, approve-as-item, merge, list-operations, delete.
  - `balances.rs`: list (filterable), by-site, summary.
  - `assets.rs`: pending-acceptance list, lost-assets list/get/resolve, issued-assets list.
  - `documents.rs`: generate, get, render, list, update-status, list-for-operation.
  - `recipients.rs`: list (searchable), create, get, update, delete, merge.
  - `reports.rs`: item-movement, stock-summary.
  - `device_sync.rs`: ping, push, pull, bootstrap/sync.
  - `health.rs`: server-info, health, ready, health-detailed, health-liveness.
  - `admin.rs`: sites list/create/update, users list/get/create/update/delete/rotate-token, devices list/get/register/update/delete/rotate-token.
- Verification: `cargo fmt --all -- --check` (clean), `cargo clippy --workspace --all-targets -- -D warnings` (clean), `cargo build --workspace` (pass), `cargo test --workspace` (8/8 pass).

## 7. Level 2 — Auth, Device Bootstrap, and Profile Context

### 7.1 Scope

Implement local identity/session context without turning core into a password-login system unless SyncServer adds a login endpoint.

### 7.2 Required behavior

- Define `TokenProvider` or equivalent host-token injection boundary.
- Implement CLI/test token provider using environment variable names only.
- Add facade methods for token binding if provider callbacks are not sufficient.
- Refresh identity through `/auth/context` and `/auth/sites`.
- Persist non-secret identity metadata: user id/name/role, available sites, permissions summary, device id, active site, server protocol info.
- Register or bind device using the ADR-selected route: admin devices, bootstrap, or existing device token injection.
- Validate active site access before site-scoped calls.
- Support logout/reset that clears cached identity and local profile state without leaking tokens.

### 7.3 Acceptance

- A profile can be opened, user token bound, identity refreshed, available sites listed, and active site persisted/reopened.
- Device id/token state is represented without storing raw token secrets unless an explicit encrypted storage ADR approves it.
- Observer/storekeeper/chief/root access differences are visible to the facade.

### 7.4 Verification

- Unit: permission and site-scope helpers.
- Component: profile reopen and active-site persistence using SQLite temp DB.
- Integration: `/auth/context`, `/auth/sites`, and selected device bootstrap route against real stand.

### 7.5 Level 2 Completion Evidence

- Created `crates/warehouse_core/src/auth/` module with:
  - `token_provider.rs`: `TokenProvider` trait, `NullTokenProvider`, `CliTokenProvider` (reads `WAREHOUSE_USER_TOKEN`/`SYNC_USER_TOKEN` and `WAREHOUSE_DEVICE_TOKEN`/`SYNC_DEVICE_TOKEN` from env).
  - `profile.rs`: `Profile` struct (user_id, name, email, role, is_root, available_sites, device_id, active_site, protocol_version, refreshed_at). Full load/save/clear via `AuthContextRepo` SQLite key-value storage.
  - `ProfileService`: `refresh_from_auth_context()`, `set_active_site()` (validates against available sites), `clear_active_site()`, `logout()` (clears all profile keys + drops client), `validate_site_access()`, `sites()`, `available_site_ids()`, `has_permission()`, `is_authenticated()`.
  - `Profile::set_active_site()` validates site access before persisting.
- Updated `CoreHandle` facade:
  - `set_token_provider()` for binding platform token injection.
  - `load_profile()` — load persisted identity from SQLite.
  - `refresh_identity()` — calls `/auth/context`, persists result.
  - `set_active_site()` / `clear_active_site()` / `logout()` — full lifecycle.
  - `bootstrap_device()` — calls `/bootstrap/sync`, builds and persists Profile.
  - `health_remote()` — real `/health` check.
  - Lazy `SyncServerClient` creation with tokens from provider.
- Verification: `cargo fmt`, `cargo clippy -- -D warnings`, `cargo build`, `cargo test` (8/8) — all pass.

## 8. Level 3 — SQLite Repositories and Migrations Complete

### 8.1 Scope

Complete repository coverage for the local working set. Existing migrations are a foundation, not yet enough by themselves.

### 8.2 Required repositories

- Profile metadata and auth context.
- Sites, units, categories, items, category tree/search indexes.
- Inventory subjects and balances.
- Pending acceptance, lost assets, issued assets.
- Recipients.
- Operation history, operation details, operation drafts, draft lines.
- Temporary items and resolution state.
- Documents and document-operation links.
- Reports cache.
- Sync cursors/runs/progress.
- Outbox events/commands/retry state.
- Conflicts and error log.

### 8.3 Required behavior

- Forward-only migrations with deterministic ordering.
- Transaction helpers for multi-table sync batches.
- Idempotent upserts for server snapshots.
- Full refresh helpers for endpoints that lack tombstones.
- Readonly open mode for diagnostics.
- Test fixture builder for seeded profiles.

### 8.4 Acceptance

- Every table used by sync/facade has a repository or documented reason why direct SQL is contained inside one service.
- Migrations run on empty DB and on previous schema without data loss in tests.
- Repositories support deterministic offline reads after seeded sync data.

### 8.5 Verification

- Unit/component repository tests with real SQLite, not mocks.
- Migration tests from schema version 0 to latest.
- Transaction rollback test for partial sync failure.

## 9. Level 4 — Pull Sync and Staged Bootstrap

### 9.1 Scope

Implement read-side sync from SyncServer to local SQLite.

### 9.2 Required services

- `BootstrapService`: first-run protocol check, identity/site/device context, initial working set.
- `PullSyncService`: incremental refresh by endpoint family.
- `CursorStore`: per-family cursor state, server time, `updated_after`, `server_seq` where applicable.
- `SnapshotWriter`: transactional writes into repositories.

### 9.3 Required pull families

1. Health/readiness/protocol.
2. Auth context and available sites.
3. Sites.
4. Catalog items/categories/units/category tree.
5. Recipients.
6. Balances for active site and allowed sites.
7. Pending acceptance, lost assets, issued assets.
8. Temporary items and related operations.
9. Operation history and operation details for configured windows/pages.
10. Document metadata and operation document links.
11. Reports on demand with parameter-hash cache.
12. Device `/pull` event stream if it provides authoritative incremental changes.

### 9.4 Required behavior

- If `/bootstrap/sync` is not enough for full offline working set, use staged bootstrap through normal read endpoints and document the missing SyncServer extension.
- Respect roles/site scopes.
- Page through large endpoints until complete or configured limit is reached.
- Mark stale data with last refresh time and failure reason.
- Store sync run summary for UI diagnostics.

### 9.5 Acceptance

- After online bootstrap/pull, the core can restart offline and serve catalog, balances, operation history, temporary items, asset registers, recipients, and document metadata from SQLite.
- Failed endpoint families do not corrupt already committed families.
- Active-site switch triggers the correct scoped refresh behavior.

### 9.6 Verification

- Component: pull services against fake clients and temp SQLite.
- Integration: real SyncServer stand with seeded catalog/balances/operations/temp/assets/documents.
- Smoke: CLI command `sync-pull` then offline CLI read commands.

## 10. Level 5 — Read-Only CoreFacade and CLI Smoke

### 10.1 Scope

Replace placeholder `CoreHandle` methods with a stable read-oriented facade. UI clients must depend on this facade, not on repositories or HTTP clients.

### 10.2 Required facade groups

- Lifecycle: `open`, `open_readonly`, `close`, `profile_status`, `health_local`, `diagnostics`.
- Connectivity: `health_remote`, `ready_remote`.
- Auth/site: `bind_user_token` or provider setup, `refresh_identity`, `get_auth_context`, `list_available_sites`, `set_active_site`, `get_active_site`.
- Sync read: `bootstrap`, `pull_once`, `get_sync_status`, `list_sync_runs`.
- Catalog: `search_items`, `get_item`, `list_units`, `list_categories`, `get_category_tree`, `get_parent_chain`.
- Balances/assets: `list_balances`, `get_balance_by_subject`, `get_balance_summary`, `list_pending_acceptance`, `list_lost_assets`, `list_issued_assets`.
- Operations read: `list_operations`, `get_operation`.
- Recipients: `search_recipients`, `get_recipient`.
- Temporary items read: `list_temporary_items`, `get_temporary_item`, `list_temporary_item_operations`.
- Documents/reports read: `list_documents`, `get_document`, `list_operation_documents`, `run_stock_summary`, `run_item_movement`.

### 10.3 Required CLI commands

- `health-remote`.
- `auth-context`.
- `site list`, `site set`.
- `bootstrap`.
- `sync-pull`.
- `catalog search`.
- `balances list`.
- `operations list`, `operations get`.
- `temp-items list`.
- `assets pending|lost|issued`.

### 10.4 Acceptance

- CLI can demonstrate the full read-only path against the real stand.
- Facade returns DTOs suitable for FFI flattening; no `sqlx` or `reqwest` types leak through.
- Read-only mobile/WPF screen contracts can be implemented from this facade.

### 10.5 Verification

- Component facade tests against fake clients and temp SQLite.
- CLI smoke tests against real SyncServer stand.
- Offline restart smoke: pull online, stop stand/network, read catalog/balances/history from local profile.

## 11. Level 6 — FFI Foundation for Read-Only Facade

### 11.1 Scope

Make the read-only facade callable from Android Kotlin and WPF C#.

### 11.2 Required binding strategy

- Kotlin Android: UniFFI unless Level 0 ADR rejects it.
- C# WPF: C ABI/PInvoke wrapper or selected generator; document memory ownership and threading.
- Do not expose internal Rust repository/client types.
- Use FFI-safe DTOs and error envelopes.
- Convert JSON-like dynamic fields into explicit strings or typed wrappers where needed.

### 11.3 Required FFI functions/methods

- Initialize logging/diagnostics without leaking secrets.
- Open/close profile.
- Bind or configure token provider for tests/host.
- Refresh identity and set active site.
- Bootstrap/pull once.
- Query sync status.
- Search catalog, list balances, list operations, list temp items, list assets.
- Return stable `CoreErrorDto` with machine-readable code and human-readable message.

### 11.4 Required build outputs

- Android `.so` targets at minimum: `arm64-v8a`, `x86_64` for emulator.
- Generated Kotlin bindings or AAR assembly instructions.
- Windows x64 DLL for WPF smoke.
- C# wrapper sample or smoke console project instructions.

### 11.5 Acceptance

- Kotlin instrumentation or JVM smoke can open a test profile and call read-only facade methods.
- C# smoke can open a test profile and call read-only facade methods.
- Memory and handle lifecycle are documented and tested.

### 11.6 Verification

- Static Rust checks plus binding generation checks.
- FFI unit/component tests for error mapping and handle lifecycle.
- Android emulator/device smoke if Android toolchain is available; otherwise unchecked with blocker note.
- WPF/C# smoke on Windows x64 if .NET toolchain is available; otherwise unchecked with blocker note.

## 12. Level 7 — OperationDraftService

### 12.1 Scope

Implement offline operation draft creation, editing, validation, and conversion to server commands.

### 12.2 Required operation types

- `RECEIVE`.
- `EXPENSE`.
- `WRITE_OFF`.
- `MOVE`.
- `ADJUSTMENT`.
- `ISSUE`.
- `ISSUE_RETURN`.

### 12.3 Required behavior

- Create, update, clone, delete local drafts.
- Edit header fields: operation type, site, source/destination site, effective date, recipient, issued-to name, comment.
- Edit lines for catalog items.
- Edit lines for inline temporary items in receive flows.
- Generate and persist `draft_id`, line IDs, and idempotency/client request identifiers where needed.
- Validate required fields by operation type.
- Validate role/site capabilities from cached auth context.
- Validate positive quantities and item/temp item presence.
- Convert a valid draft to `OperationCreate` or an outbox command payload.
- Keep server-rejected drafts user-editable.

### 12.4 Acceptance

- A user can create a complete draft offline for each operation type supported by policy.
- Draft validation matches SyncServer expectations for required fields as closely as possible.
- Drafts survive process restart and profile reopen.
- Core still does not mutate local balances as final truth.

### 12.5 Verification

- Unit: validation matrix for all operation types.
- Component: draft repository and service tests with SQLite.
- Contract: valid draft serialized payload is accepted by real SyncServer when sent online.
- Negative tests: missing site, invalid move sites, invalid qty, permission denied, temp item misuse.

## 13. Level 8 — Outbox and Push Transport

### 13.1 Scope

Implement durable command queue for offline user intent.

### 13.2 Required model

Outbox entries must include:

- local outbox id;
- event/command UUID;
- command type;
- payload version;
- target site;
- user/device identity snapshot;
- idempotency key or `client_request_id`;
- state: pending, ready, sending, accepted, duplicate, rejected, conflict, dead-letter, cancelled;
- retry count, next retry time, last error;
- server operation/document/temp item ids when accepted.

### 13.3 Required transports

- `DevicePushTransport` for `/push` if the ADR confirms SyncServer accepts the needed operation events.
- `HttpCommandOutboxTransport` for replaying normal user-token endpoints if `/push` is not yet semantically complete.
- Transport selection must be explicit and testable, not hidden in UI clients.

### 13.4 Required behavior

- Queue draft submit.
- Queue operation update/cancel/submit/accept-lines if policy allows offline queueing.
- Queue temporary-item moderation only if Level 0 policy allows it; otherwise force online.
- Retry network failures with backoff.
- Classify duplicate-same-payload as accepted/duplicate.
- Classify UUID collision or payload mismatch as conflict.
- Preserve validation/server rejection for user correction.
- Never drop failed user intent silently.

### 13.5 Acceptance

- Offline draft can be queued without network.
- When network returns, command is sent exactly once or safely deduplicated.
- Rejected command remains visible and editable.
- Conflict state contains enough data for UI to explain and recover.

### 13.6 Verification

- Unit: state transitions and retry/backoff.
- Component: outbox with fake transport and SQLite.
- Integration: real stand accepts queued operation; duplicate send is idempotent or produces documented duplicate handling.
- Failure tests: timeout, 401/403, 409, validation 422, server unavailable.

## 14. Level 9 — SyncEngine Orchestration

### 14.1 Scope

Build the orchestrator used by CLI, Android WorkManager, and WPF background jobs.

### 14.2 Required behavior

- Single-process sync lock per profile.
- Optional cross-process lock if multiple hosts may open the same profile.
- Sync modes: bootstrap, pull-only, push-only, push-then-pull, full.
- Cancellation token support.
- Progress events suitable for FFI/UI.
- `ping` before heavy sync when device token exists.
- Backoff and server throttle handling.
- Active-site aware sync.
- Safe resume after crash during sending.
- Conflict collection and summary.

### 14.3 Acceptance

- Concurrent sync requests do not corrupt state.
- Push-then-pull updates operation history and balances after accepted commands.
- Cancellation leaves profile in a resumable state.
- UI clients can display current sync phase, progress, last success, and last error.

### 14.4 Verification

- Unit: state machine and conflict classification.
- Component: fake transport/client with deterministic progress.
- Integration: real stand full sync with queued outbox.
- Chaos: interruption between send and acknowledgment, retry after restart.

## 15. Level 10 — Full Client Facade Through FFI

### 15.1 Scope

Expose complete client workflows through the stable facade and bindings. After this level, Android/WPF should not need surrogate domain behavior for core-supported flows.

### 15.2 Required additional facade methods

- Drafts: create, update header, add item line, add temporary item line, update line, delete line, validate, delete draft, list drafts, get draft.
- Submit: queue draft submit, submit now when online, retry outbox event, cancel outbox event.
- Outbox: list events, get event, explain event state.
- Sync: sync once, subscribe/read progress, get conflicts, resolve or acknowledge conflict where supported.
- Temporary items: approve-as-item, merge, delete, with online/offline policy enforced by core.
- Assets: accept pending lines and resolve lost assets if supported offline/online by policy.
- Documents: generate document and fetch render metadata/bytes according to cache policy.
- Reports: run online refresh and read cached result.

### 15.3 Required FFI expansion

- All methods above exposed to Kotlin and C# or explicitly marked unavailable with reason.
- FFI error codes cover validation, auth, forbidden, not found, conflict, unavailable, timeout, sync locked, cancelled, protocol mismatch.
- Long-running operations return progress or observable run id; UI thread must not be blocked.

### 15.4 Acceptance

- Android Kotlin smoke can execute: open -> bind token -> bootstrap -> search item -> create draft offline -> queue -> sync -> observe accepted/rejected.
- C# smoke can execute the same core scenario without direct HTTP/SQLite.
- Surrogate implementations can be replaced for supported flows by one DI/module switch in clients.

### 15.5 Verification

- FFI smoke tests for Android and Windows.
- Real stand full scenario through FFI, not only through Rust unit tests.
- Regression: read-only methods from Level 6 still pass unchanged.

## 16. Level 11 — Contract, Performance, Chaos, and Regression Hardening

### 16.1 Scope

Prove the core remains compatible with SyncServer and robust under realistic data/failure sizes.

### 16.2 Required hardening packs

- Contract test pack for every required endpoint group.
- Migration compatibility pack from old profile schema to latest.
- Performance pack:
  - 10k catalog items;
  - 1k categories;
  - 10k balances;
  - 1k operation history rows;
  - 1k outbox entries.
- Network chaos pack:
  - timeout;
  - connection refused;
  - partial response/serialization error;
  - auth token expired;
  - server 409/422;
  - backoff requested by server.
- Secret-safety pack: logs and error messages contain no token values.

### 16.3 Acceptance

- Contract tests fail loudly when SyncServer schema changes incompatibly.
- Sync remains resumable after crash/interruption.
- Query latency is documented for large local datasets.
- No critical flow depends on mocked-only tests.

### 16.4 Verification

- `cargo test --workspace`.
- Ignored/feature-gated real stand contract tests.
- Performance test report with machine/date/profile size.
- QA review of failure logs and secret redaction.

## 17. Level 12 — Documentation, Packaging, and Client Handoff

### 17.1 Scope

Prepare the core as a reusable product for Android and WPF teams.

### 17.2 Deliverables

- `README.md` update for build/test/package commands.
- `docs/CORE_FACADE_V1.md` final method/DTO/error reference.
- `docs/ANDROID_BINDINGS.md` with Gradle/NDK/AAR instructions.
- `docs/WPF_BINDINGS.md` with DLL/PInvoke/NuGet or wrapper instructions.
- `docs/SYNC_STAND.md` with real stand setup.
- `docs/CLIENT_HANDOFF.md` with integration checklist.
- Changelog entry for core facade version.
- Example profiles/fixtures with no secrets.

### 17.3 Acceptance

- A mobile agent can wire `RustCoreHandle` using documented artifacts without reading internal Rust modules.
- A WPF agent can wire the C# wrapper using documented artifacts without direct SyncServer access.
- All active docs agree that clients depend on Rust core for warehouse domain behavior.

### 17.4 Verification

- Docs-curator review.
- QA follows Android/WPF handoff instructions on a clean checkout or documents blockers.
- Final evidence table complete.

## 18. Required Test Ladder

### 18.1 Static checks

Required for every code level:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-run
```

### 18.2 Unit tests

Required coverage:

- DTO serde/mapping.
- HTTP header construction and secret redaction.
- Error classification.
- Auth/site permission helpers.
- Draft validation.
- Outbox state transitions.
- Sync state machine.
- Conflict classification.
- FFI error envelope conversion.

### 18.3 Component tests

Required coverage:

- Repositories against real SQLite temp DB.
- Facade against fake SyncServer clients.
- HTTP client against local mock HTTP server.
- CLI commands against temp profiles.
- FFI handle lifecycle and nullability boundaries.

### 18.4 Integration tests with real dependencies

Required coverage:

- Real SQLite migrated schema.
- Real SyncServer test stand.
- Real PostgreSQL test database for SyncServer.
- Real HTTP calls through `reqwest`, not mocked.
- Real profile directory lifecycle.

### 18.5 Real stand smoke tests

See Section 19. Stand smoke is mandatory for Levels 1, 2, 4, 5, 8, 9, 10, 11, and 12.

### 18.6 UI automation applicability

Core itself has no UI, so Playwright/FlaUI are not directly applicable inside this TZ.

However:

- Android FFI smoke/instrumentation is required when Android bindings are touched.
- WPF/C# smoke is required when C# bindings are touched.
- Later `WarehouseMobile` and `WarehouseDesktop` TZ files must add Android UI automation and FlaUI scenarios using this core.
- The checklist item `6. UI automation tests complete` may be checked for this TZ only after the required binding smoke/instrumentation evidence is attached, or left unchecked with a reason if client UI automation belongs to a downstream TZ.

### 18.7 User scenarios

Required scenarios:

1. First launch: open profile, bind user token, refresh identity, choose active site, bootstrap.
2. Offline browse: pull catalog, stop stand/network, search item/category/unit locally.
3. Balances: pull balances/assets, read by active site offline.
4. Operation draft: create/edit/validate draft offline.
5. Outbox: queue draft, reconnect, push, pull updated operation history/balances.
6. Temporary item receive: create receive draft with inline temporary item and idempotency key.
7. Temp moderation: approve-as-item or merge according to policy.
8. Assets: pending acceptance and lost asset resolution according to policy.
9. Documents: generate document and fetch/render metadata or bytes.
10. Reports: stock summary and item movement refresh/cache.
11. Conflict: duplicate send, validation rejection, or UUID collision yields user-visible recoverable state.

### 18.8 Regression pack

Regression must include:

- auth and access scope;
- active site switching;
- catalog tree/search;
- subject-first balances;
- operation workflow;
- temporary item flows;
- pending/lost/issued asset flows;
- documents and reports;
- outbox idempotency;
- sync locking/cancellation;
- FFI handle lifecycle;
- secret redaction in logs.

## 19. Real Test Stand

### 19.1 Database type and lifecycle

- SyncServer stand uses PostgreSQL test database.
- Database is created/reset per stand run or per test suite.
- Alembic migrations must be applied from the SyncServer project.
- Seed data must be deterministic and safe to delete.

### 19.2 Required seed data

- Users/tokens by role: root, chief storekeeper, storekeeper, observer.
- At least two active sites.
- Access scopes for non-root users.
- Registered device with device token.
- Units, categories, category tree, items with SKU/search data.
- Recipients.
- Balances created through server operations, not direct DB writes where possible.
- Operation history for all supported operation types where feasible.
- Temporary item created by receive flow.
- Pending acceptance row.
- Lost asset row and issued asset row.
- Document template/data sufficient for document generation.

### 19.3 Services to start

- PostgreSQL test database.
- SyncServer FastAPI app against that database.
- Optional mock object/file storage only if documents require it.
- `warehouse_cli` and Rust tests executed against the stand.

Use background PTY sessions for long-running services.

### 19.4 Environment variable names only

Never commit values. Use names only:

- `SYNC_TEST_DATABASE_URL`
- `SYNC_TEST_BASE_URL`
- `SYNC_TEST_ROOT_USER_TOKEN`
- `SYNC_TEST_CHIEF_USER_TOKEN`
- `SYNC_TEST_STOREKEEPER_USER_TOKEN`
- `SYNC_TEST_OBSERVER_USER_TOKEN`
- `SYNC_TEST_DEVICE_TOKEN`
- `SYNC_TEST_DEVICE_ID`
- `SYNC_TEST_SITE_ID_MAIN`
- `SYNC_TEST_SITE_ID_SECONDARY`
- `WAREHOUSE_CORE_TEST_PROFILE_DIR`
- `WAREHOUSE_CORE_CONTRACT_TESTS`

### 19.5 Health checks

- `GET /api/v1/health` returns healthy status.
- `GET /api/v1/ready` returns DB ready.
- `GET /api/openapi.json` is available for contract comparison.
- Core CLI `health-remote` succeeds.

### 19.6 Smoke commands to implement and run

Command names may be adjusted by implementers, but equivalent smoke coverage is mandatory:

```bash
cargo run -p warehouse_cli -- health-remote
cargo run -p warehouse_cli -- auth-context
cargo run -p warehouse_cli -- bootstrap
cargo run -p warehouse_cli -- sync-pull
cargo run -p warehouse_cli -- catalog search --query bolt
cargo run -p warehouse_cli -- balances list
cargo run -p warehouse_cli -- operations draft-create --type RECEIVE
cargo run -p warehouse_cli -- operations draft-submit --draft-id <test-draft-id>
cargo run -p warehouse_cli -- outbox sync
cargo run -p warehouse_cli -- operations list
```

### 19.7 Reset and cleanup

- Stop SyncServer background session.
- Drop/reset PostgreSQL test database/schema.
- Delete `WAREHOUSE_CORE_TEST_PROFILE_DIR`.
- Delete temporary FFI build artifacts not meant to be committed.
- Preserve logs/screenshots only in documented test-output paths.

## 20. Evidence Table Template

Executor completion reports must include this table:

| Check | Command / Tool | Result | Evidence |
|---|---|---|---|
| Static checks | `cargo fmt`, `cargo clippy`, `cargo test --no-run` | pass/fail/skipped | log path or summary |
| Unit tests | `cargo test --workspace` | pass/fail/skipped | log path or summary |
| Component tests | SQLite/mock HTTP/CLI tests | pass/fail/skipped | test names/log path |
| DB integration | real SQLite + migrated schema | pass/fail/skipped | profile/fixture note |
| SyncServer integration | real stand contract tests | pass/fail/skipped | stand URL/log path, no secrets |
| Stand smoke | CLI smoke commands | pass/fail/skipped | command output/log path |
| FFI smoke | Android/C# smoke | pass/fail/skipped | artifact/log path |
| User scenarios | scenario list | pass/fail/skipped | notes/logs/screenshots |
| Regression | regression pack | pass/fail/skipped | report path |

## 21. Task Routing Guidance

- Use `offline-core` for Rust core implementation levels.
- Use `syncserver-backend` only when a missing SyncServer contract/endpoint/semantic blocks the core.
- Use `qa-verifier` for real stand, contract tests, and evidence review.
- Use `docs-curator` for ADR/docs updates.
- Do not route Android/WPF UI work from this TZ except binding smoke required to prove FFI.

## 22. Known Open Questions and Blockers

These must be answered in Level 0 or documented as blockers:

1. Does SyncServer have a real user login endpoint, or is client auth strictly token binding from an external/admin provisioning flow?
2. Does `/bootstrap/sync` return enough data for offline first-run, or must core always use staged bootstrap via read endpoints?
3. Does `/push` support all operation command semantics needed by offline clients, or should outbox replay normal operation endpoints first?
4. Which endpoints expose tombstones/deletions for incremental sync? If absent, what full-refresh cadence is acceptable?
5. Are catalog/admin writes allowed offline, or online-only due to global conflict risk?
6. Which document bytes may be cached locally, for how long, and with which platform policy?
7. Which admin APIs are packaged into mobile clients, if any?
8. Which C# binding generator/path is the maintained one for WPF?

## 23. Final Acceptance Criteria

This TZ is complete only when:

- Levels 0 through 12 are complete or any skipped scope is explicitly removed by a new ADR/TZ.
- No client-required warehouse domain flow depends on direct SyncServer calls from Android/WPF.
- The real stand scenario proves online bootstrap, offline reads, offline draft creation, outbox push, pull refresh, and conflict/rejection handling.
- Kotlin and C# can call the supported facade through generated bindings/wrappers.
- Documentation tells client teams exactly how to integrate the core.
- QA verifier checks checklist item `10. Final acceptance review complete` after evidence review.
