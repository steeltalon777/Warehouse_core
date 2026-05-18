# TZ: Warehouse_client_core Capability Plan

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

- Architect owns this plan and checklist structure.
- Executor agents may check implementation and test items only after completing the work and attaching verification evidence.
- QA verifier may check final acceptance only after reviewing command output, stand notes, and scenario evidence.
- Unavailable stand/UI checks stay unchecked with a blocker note; they must not be replaced by unit tests.

## 0. Purpose

This document defines what `Warehouse_client_core` must eventually provide after reviewing current `SyncServer` and `Warehouse_web` functionality.

The core is the shared Rust offline-first runtime for future WPF, MAUI, Android, and possible Swift shells. UI clients should become thin platform shells; the core owns local storage, offline intent, sync, DTO mapping, validation, and safe AI/tool access.

## 1. Architecture Boundaries

### 1.1 Source of Truth

- `SyncServer` remains the authoritative source for warehouse domain data, access rules, operation workflow, balances, documents, reports, temporary item moderation, and final validation.
- `Warehouse_client_core` may cache server state and local user intent, but must not become a second backend.
- Local SQLite is a working set, outbox, and audit cache, not the final source of truth.

### 1.2 Client Roles

- WPF/MAUI/Kotlin/Swift own UI, native permissions, platform secure storage, camera/scanner integration, windowing, and platform-specific background scheduling.
- Rust core owns shared warehouse behavior: data model, local schema, outbox, sync, validation, conflict handling, tool contracts, and stable facade APIs.
- Desktop may host MCP/REST/JSON-RPC for AI, but MCP tool logic and policy must live in Rust core.
- Browser Angular continues to call Django BFF; Rust core does not replace Django web.

### 1.3 Forbidden Ownership

- Core must not connect directly to the SyncServer database.
- Core must not store SyncServer root/admin secrets in source or logs.
- Core must not implement WPF/Android UI logic.
- Core must not allow AI direct database access; AI uses allowlisted tools only.

## 2. Evidence From Current Systems

### 2.1 SyncServer Capability Inventory

Current authoritative APIs and services found in `SyncServer/app/api/` and service modules:

| Area | SyncServer capability | Core implication |
|---|---|---|
| Auth/access | `POST /auth/sync-user`, `GET /auth/me`, `/sites`, `/context`; roles `root`, `chief_storekeeper`, `storekeeper`, `observer`; site-scoped access | Core needs identity context, role/site scope cache, token/device auth headers, access-aware facade methods |
| Catalog read | items, categories, units, sites, category tree, browse/read endpoints, parent chains | Core needs full catalog cache, tree navigation, search indexes, stale marker, paging/filter DTOs |
| Catalog admin | CRUD units/categories/items, bulk units/categories, soft delete/archive semantics | Core needs online command wrappers and optional offline admin drafts if explicitly allowed by policy |
| Operations | list/detail/create/update/effective date/submit/cancel/accept-lines | Core needs operation drafts, idempotent submit outbox, workflow state, acceptance flows, rollback/error states |
| Balances | list/by-site/summary; subject-first inventory model | Core needs balance snapshot cache, local read models, subject-first DTOs, no direct balance writes |
| Assets | pending acceptance, lost assets, resolve lost asset, issued assets | Core needs asset register cache, acceptance/lost resolution workflows, conflict-aware pending states |
| Temporary items | list/detail/operations/approve-as-item/merge/delete; inline temp receive with `client_request_id` | Core needs temp item drafts, moderation views, idempotent request IDs, resolution state tracking |
| Recipients | CRUD and merge | Core needs recipient cache and operation form support |
| Documents | generate, read, render HTML/PDF, list, status, operation document links | Core needs document metadata cache and server-render fetch; UI owns display/print integration |
| Reports | item movement, stock summary | Core needs report query DTOs, cached last results, AI-safe report tools |
| Device sync | `/ping`, `/push`, `/pull`, `/bootstrap/sync`; event UUID idempotency; `server_seq` cursor | Core needs sync engine, outbox, pull cursor, duplicate/collision handling, bootstrap lifecycle |
| Admin | users, scopes, devices, sites, roles, token rotation | Core may expose admin clients for desktop admin shells, but should keep normal offline clients least-privilege |
| Health | `/health`, `/ready`, detailed readiness/liveness | Core needs stand diagnostics and connectivity state |

Important contracts:

- Operations are server-authoritative commands; submitted effects and balances are calculated by SyncServer services.
- Balances and reports are derived server state; core caches them but never edits them directly.
- Inventory is subject-first: core DTOs must support item subjects, temporary item subjects, resolved item IDs, display names, and old item-id compatibility only as a mapped field.
- Sync push must be idempotent by event UUID and classify duplicate-same-payload vs UUID collision.
- Bootstrap currently appears limited; full offline bootstrap may require SyncServer extensions or a core-side staged bootstrap using existing read endpoints.

### 2.2 Warehouse_web / Django Capability Inventory

Current active web workflows found in `Warehouse_web/apps/*` and `apps/sync_client/`:

| Area | Django capability | Core implication |
|---|---|---|
| Session binding | Django login binds SyncServer identity into session; active site/default site stored in session/binding | Core needs equivalent local profile/session state, active site switching, and identity refresh APIs |
| Sync client wrappers | auth, catalog, operations, balances, assets, temp items, documents, recipients, admin/access wrappers | These wrappers are a useful contract map for core HTTP client modules |
| Catalog UI | SSR catalog browse; nomenclature management via SSR fallback and Angular host | Core needs catalog browse/admin facade for native shells and future Angular parity reference |
| Operations UI | create/list/detail/submit/cancel, item search JSON, temporary item draft helper, pending acceptance, lost asset resolution | Core needs operation form models, draft validation, search, submit/cancel/acceptance/lost flows |
| Balances UI | list, summary, by-site | Core needs fast offline balance read models and online refresh |
| Temporary items UI | list/detail/approve/merge/search/delete/bulk delete | Core needs moderation facade and conflict state for offline/online transitions |
| Documents UI | generate operation document, redirect/stream server-rendered PDF | Core needs document generation/fetch facade; platform shell handles rendering/opening/printing |
| Angular shell | `/nomenclature/` hosted by Django; BFF namespace is still incomplete | Core should not copy browser BFF, but should expose native capability parity for nomenclature workflows |
| Local state | Django stores auth/session/bindings/catalog cache only | Core should mirror this principle: technical local state and cache, not domain authority |

Observed Django gaps that affect core planning:

- Current Django tests are mostly mocked/unit/view tests; core must define real SyncServer stand checks early.
- Angular BFF for nomenclature is not yet a broad API surface; core should derive contracts from SyncServer first, using Django workflows as UX parity requirements.
- Some Django tests appear stale against current code paths; core contracts should be generated from SyncServer schemas/endpoints, not only Django expectations.

## 3. Target Core Product Capabilities

### 3.1 Local Profile, Auth, and Access Context

Core must provide:

- local client profile creation/open/migrate/reset;
- server URL and device/user identity metadata without owning secret storage;
- token injection hooks supplied by platform secure storage;
- `auth_context_refresh` equivalent to SyncServer `/auth/context`;
- active site selection and default-site persistence;
- role and scope checks for facade methods;
- offline capability flags: what can be viewed, drafted, queued, or must be online.

Minimum facade examples:

- `open_profile(path, config)`
- `set_server_endpoint(url)`
- `refresh_identity()`
- `list_available_sites()`
- `set_active_site(site_id)`
- `get_access_context()`

### 3.2 Local SQLite Storage and Migrations

Core must own one normalized SQLite schema for all native clients:

- `profile_metadata`, `schema_migrations`, `sync_state`;
- `sites`, `access_scopes`, `users/devices metadata` as cache only;
- `catalog_categories`, `catalog_items`, `catalog_units`, category tree/path indexes;
- `recipients`;
- `inventory_subjects`, `balances`, `balance_snapshots`, `asset_registers`;
- `operations`, `operation_lines`, `operation_drafts`, `operation_workflow_state`;
- `temporary_items`, temp item operation links and resolution state;
- `documents`, document operation links, render cache metadata;
- `reports_cache` for last report results and parameters;
- `outbox_events`, `outbox_commands`, retry metadata, idempotency keys;
- `conflicts`, `sync_errors`, `audit_log`, `tool_invocations`.

Storage must support:

- migrations with forward-only versioning;
- transactional repository APIs;
- readonly open mode for diagnostics/AI analysis;
- profile reset/export diagnostics without secrets;
- deterministic test fixtures.

### 3.3 SyncServer HTTP Contract Layer

Core must implement typed clients mirroring SyncServer, grouped by bounded area:

- auth/access client;
- catalog read/admin client;
- recipients client;
- operations client;
- balances client;
- asset register client;
- temporary item client;
- documents client;
- reports client;
- device sync client;
- admin client only for explicitly admin-capable shells.

Rules:

- All DTOs must be generated or hand-mapped from SyncServer schemas with version notes.
- HTTP errors must map to stable core error classes: unauthenticated, forbidden, validation, conflict, backend unavailable, timeout, protocol mismatch.
- Core must expose health/readiness diagnostics for UI and QA stands.
- No browser/Django session assumptions in core; only SyncServer tokens supplied by host.

### 3.4 Bootstrap and Pull Sync

Core must support first-run and incremental sync:

- health check and protocol/version negotiation;
- identity/context refresh;
- site/access bootstrap;
- catalog full refresh and incremental refresh strategy;
- recipient refresh;
- balances and asset register refresh per active site and all allowed sites if role allows;
- temporary item moderation refresh;
- operation history refresh with paging/windows;
- document metadata refresh;
- report refresh on demand;
- sync cursor persistence for `/pull` when server-side event stream is sufficient.

If SyncServer bootstrap lacks full warehouse payloads, core should implement staged bootstrap via existing read endpoints and document the required future SyncServer extension.

### 3.5 Outbox, Push Sync, and Idempotency

Core must own an outbox for local user intent:

- command/event UUID generation;
- `client_request_id` for temporary item receive flows;
- queued operation create/update/submit/cancel/accept-lines;
- queued temp item moderation only if policy allows offline moderation;
- retry/backoff and resumable failure state;
- duplicate-same-payload handling;
- UUID collision conflict state;
- user-visible pending/sent/accepted/rejected state.

Outbox commands are not final warehouse changes until accepted by SyncServer.

### 3.6 Catalog and Nomenclature

Core must support the full catalog/nomenclature scope required by Django and future native shells:

- list/search/filter items with paging;
- category tree, children, parent chain;
- units and sites lookup;
- uncategorized bucket semantics;
- active/inactive/archive state display;
- item/category/unit create/update/delete through SyncServer admin endpoints;
- bulk create where SyncServer supports it;
- local search indexes and stale markers;
- item balances lookup by subject/site where available.

Offline writes to catalog should be phase-gated and policy-controlled because catalog conflicts affect all clients.

### 3.7 Operations Workflow

Core must cover all SyncServer operation types:

- `RECEIVE`, `EXPENSE`, `WRITE_OFF`, `MOVE`, `ADJUSTMENT`, `ISSUE`, `ISSUE_RETURN`.

Core must provide:

- local operation draft CRUD;
- line editor models for catalog items and temporary receive items;
- validation aligned with SyncServer schemas and policies;
- source/destination site requirements by operation type;
- effective date handling;
- submit/cancel/update commands;
- pending acceptance and accept-lines flow;
- lost asset resolution flow;
- operation history and detail read models;
- clear server-rejected state and re-edit workflow.

Core must never locally apply final balance effects as truth. It may calculate previews for UX, marked as preview only.

### 3.8 Balances, Assets, and Inventory Subjects

Core must implement subject-first inventory views:

- balance list by active site and allowed sites;
- balance summary;
- per-subject and per-item lookup;
- display name and resolved item mapping;
- temporary item balance display;
- pending acceptance register;
- lost assets register;
- issued assets register;
- stale/offline indicators and last refresh time.

Core can expose local availability previews for drafting, but SyncServer remains final validator.

### 3.9 Temporary Items

Core must support:

- creating receive drafts with inline temporary item payloads;
- generating/storing `client_request_id` for idempotency;
- listing and viewing temporary items;
- showing related operations;
- approve-as-item flow;
- merge flow;
- delete flow;
- blocking/conflict states when SyncServer refuses resolution due to active registers;
- mapping temp item subjects in balances, reports, and operation history.

### 3.10 Documents and Reports

Core must support:

- generating server documents for operations;
- reading document metadata;
- fetching render bytes or URLs for HTML/PDF;
- listing operation-linked documents;
- changing document status where allowed;
- caching render metadata and optional bytes according to platform policy;
- item movement report queries;
- stock summary queries;
- report result caching with parameter hash and refresh time.

Platform shells own PDF viewer, printer integration, filesystem export, and share sheets.

### 3.11 Admin and Device Management

Core should include an optional admin feature set:

- list/create/update users;
- scopes management;
- device list/create/update/delete/rotate-token;
- site management;
- roles discovery.

This must be compile/runtime feature-gated so normal mobile/desktop users do not receive unnecessary admin surface.

### 3.12 AI Tool Gateway and MCP-Ready Contracts

Core must define safe AI/tool contracts, even if only desktop hosts MCP:

- allowlisted read tools for catalog, balances, operations, temporary items, documents, and reports;
- tool policy enforcing role/site scope;
- readonly transaction mode by default;
- audit log for every tool invocation;
- explicit human approval envelope for any write/draft action;
- prompt-injection-safe tool descriptions and bounded paging;
- no raw SQL tool exposed to AI.

Candidate core tools:

- `get_auth_context`
- `search_items`
- `search_categories`
- `list_units`
- `list_sites`
- `get_balances`
- `get_stock_summary`
- `get_item_movement`
- `get_operation_history`
- `get_operation_detail`
- `list_temporary_items`
- `list_pending_acceptance`
- `list_lost_assets`
- `explain_subject_state`

Desktop-specific host options:

- WPF launches `warehouse-core-mcp.exe` or embedded sidecar.
- WPF stores desktop secrets and handles user approval dialogs.
- Mobile reuses the same tool logic internally but does not host MCP by default.

### 3.13 Facade API for UI Clients

Core must expose a stable UI-oriented facade, not raw repositories:

- profile/session facade;
- sync facade;
- catalog facade;
- operations facade;
- balances/assets facade;
- temporary items facade;
- documents/reports facade;
- admin facade behind feature flag;
- AI tools facade behind feature flag.

FFI-facing DTOs must be stable, versioned, nullable-safe, and language-friendly for C#, Kotlin, MAUI, and possible Swift.

## 4. Implementation Levels

### Level 0 — Contract Capture and ADRs

Deliverables:

- SyncServer endpoint inventory copied into core docs with schema/version notes.
- ADR for core integration mode: sidecar JSON-RPC/REST first vs FFI first.
- ADR for SQLite library and migration strategy.
- ADR for MCP/tool split: desktop host, Rust tool policy/logic.

Acceptance:

- No production implementation starts before ADRs are merged.
- All contracts point to source files or OpenAPI/schema references.

### Level 1 — Core Skeleton and Local Profile

Deliverables:

- Workspace modules finalized: `auth`, `catalog`, `operations`, `balances`, `assets`, `temporary_items`, `documents`, `reports`, `sync`, `tools`, `facade`.
- SQLite migration runner and profile metadata.
- Typed error envelope and config model.
- CLI diagnostics: open profile, migrate, health-local.

Acceptance:

- Profile can be created, migrated, reopened, and reset in tests.
- CLI works without UI.

### Level 2 — SyncServer Typed Clients

Deliverables:

- Auth/access, catalog, balances, operations, temp items, documents, reports clients.
- Response/error mapping and request ID/correlation logging.
- Contract tests using recorded fixtures and real stand smoke tests.

Acceptance:

- Core can call SyncServer health/auth/catalog against a real test stand.
- DTO mapping handles subject-first inventory fields.

### Level 3 — Read-Only Offline Working Set

Deliverables:

- Catalog/site/access/balance/assets/temp item/document/report caches.
- Search indexes and stale markers.
- Pull refresh orchestration.

Acceptance:

- After online refresh, core can serve catalog/balances/operation history offline from SQLite.

### Level 4 — Operation Drafts and Outbox

Deliverables:

- Local operation draft CRUD.
- Offline validation.
- Outbox command model and idempotency keys.
- Online submit/cancel/update/accept-lines command execution.

Acceptance:

- Draft can be created offline, queued, then submitted after connectivity returns.
- Rejected command preserves user-editable state.

### Level 5 — Temporary Items, Assets, Documents, Reports

Deliverables:

- Temp item moderation flows.
- Pending/lost/issued asset flows.
- Document generation/fetch/status facade.
- Report query/cache facade.

Acceptance:

- Native shell can reproduce Django workflows for temp items, acceptance, lost assets, documents, and reports through core.

### Level 6 — Admin Feature Gate

Deliverables:

- Optional admin clients/facade.
- Runtime role/scope guard.
- Feature flag for packaging.

Acceptance:

- Admin APIs are unavailable in non-admin runtime profile and denied by policy.

### Level 7 — AI Tool Gateway / MCP-Ready Layer

Deliverables:

- Tool registry and executor in Rust core.
- Policy/audit/read-only transaction support.
- Desktop sidecar contract for MCP/REST/JSON-RPC host.

Acceptance:

- AI can analyze warehouse state only through scoped allowlisted tools.
- Write/draft tools require explicit approval envelope and audit evidence.

## 5. Required Test Strategy

### 5.1 Static Checks

Commands:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-run
```

Required for every implementation level.

### 5.2 Unit Tests

Cover:

- DTO mapping;
- validation rules;
- error classification;
- outbox state transitions;
- conflict classification;
- tool policy decisions.

### 5.3 Component Tests

Cover:

- repositories against SQLite;
- facade methods against fake typed clients;
- CLI commands against temp profiles;
- FFI DTO serialization and nullability boundaries.

### 5.4 Integration Tests With Real Dependencies

Use real SQLite and migrated schema for every storage/sync test.

Use a real SyncServer test stand for contract smoke suites, not only mocked HTTP.

### 5.5 Real Stand Smoke Tests

Minimum stand:

- PostgreSQL test database for SyncServer, migrated with Alembic.
- SyncServer FastAPI app started against the test database.
- Seeded root user token, device token, sites, users/scopes, catalog, balances through operations, temp items, documents.
- `Warehouse_client_core` CLI/profile opened against that stand.

Environment variable names only:

- `DATABASE_URL_TEST`
- `SYNC_TEST_BASE_URL`
- `SYNC_TEST_ROOT_TOKEN`
- `SYNC_TEST_DEVICE_TOKEN`
- `WAREHOUSE_CORE_TEST_PROFILE_DIR`

Smoke commands to add:

```bash
cargo run -p warehouse_cli -- health-remote
cargo run -p warehouse_cli -- bootstrap
cargo run -p warehouse_cli -- sync-pull
cargo run -p warehouse_cli -- smoke-catalog
cargo run -p warehouse_cli -- smoke-operation-draft-submit
```

Cleanup:

- Drop/reset PostgreSQL test schema/database.
- Delete temporary core profile directory.
- Stop SyncServer background session.

### 5.6 UI Automation

Core itself has no UI, but UI shells using core must prove integration:

- WPF: FlaUI smoke and user scenarios through core-backed profile.
- Web: Playwright remains for Django/Angular, not core.
- Android/MAUI/Swift: platform automation later, using the same core fixtures.

### 5.7 User Scenarios

Required end-to-end scenarios for core-backed clients:

1. First launch, bind identity, choose active site, bootstrap catalog.
2. Browse/search catalog offline after refresh.
3. Receive operation with catalog item, submit online, refresh balances.
4. Receive operation with temporary item and `client_request_id`, then approve/merge temp item.
5. Move or issue operation, accept lines, resolve lost asset.
6. Generate and fetch operation document.
7. Run stock summary and item movement report.
8. Disconnect network, create draft, reconnect, push outbox, handle rejection/conflict.
9. AI/read tool asks for balances and operation history with role/site scope enforced.

### 5.8 Regression Pack

Regression must include:

- auth and access scope;
- catalog tree/search;
- operation workflow;
- balances subject-first mapping;
- temp item moderation;
- assets acceptance/lost flow;
- documents render metadata;
- reports;
- sync idempotency and conflict handling;
- AI tool audit/policy.

## 6. Open Questions / Required SyncServer Extensions

- Does `/bootstrap/sync` need to return full catalog/balance/operation working set for offline clients, or should core perform staged bootstrap using existing read endpoints?
- Are catalog admin writes allowed offline, or online-only due to global impact?
- Should operations offline push use existing business endpoints or device `/push` events with server-side event processors?
- Which document bytes may be cached locally and for how long?
- What is the canonical OpenAPI/schema generation workflow for Rust DTOs?
- Which admin APIs are safe to package into mobile shells?
- What exact sidecar protocol should desktop use first: JSON-RPC over stdio, local HTTP, named pipe, or direct FFI?

## 7. First Recommended TZ After This Plan

Create a focused ADR/TZ for Level 0:

- choose sidecar vs FFI-first integration;
- choose SQLite/migration crate;
- define SyncServer contract source and DTO generation policy;
- define core profile/test stand layout;
- define MCP/tool split and audit envelope.

No broad implementation should start before Level 0 decisions are captured.
