# Changelog

## [0.3.0] — Client-Ready Completion

### Added
- Levels 1–6: SyncServer HTTP client (13 modules), Auth/Profile/Bootstrap, SQLite repos + migrations (5 → 6), Pull Sync (12 families), Read-only CoreFacade + CLI, FFI foundation (48 extern "C" functions)
- Level 7: OperationDraftService — offline draft CRUD, validation, to_operation_create conversion
- Level 8: OutboxService — durable command queue, idempotency, retry with backoff, conflict/dead-letter classification
- Level 9: SyncEngine — 5-sync-mode orchestrator with progress callback, lock, conflict collection
- Level 10: Full facade through FFI — 92 CoreHandle methods, 48 extern "C" exports
- Issue Objects: domain DTOs, HTTP client, facade, FFI, CLI commands
- Stage 1: DELETE operation endpoint, catalog audit fields, bulk catalog endpoints, document render verification
- CLI: 14 command groups, 30+ subcommands

### Changed
- Migration 0005: Asset operation_id/operation_line_id INTEGER→TEXT
- Migration 0006: Sync run enrichment (families_json, mode columns)

### Fixed
- PendingAcceptanceRow/IssuedAssetRow DTO alignment with SyncServer API
- pull_recipients 404 handling, pull_device_events auth error handling
- Sync run persistence (was stub)
- Various serde alignment fixes

## [0.2.0] — Foundation

- Domain DTOs for catalog, balances, operations, assets, recipients, documents, temporary items
- CoreConfig, CoreError, CoreHandle skeleton
- SQLite migration runner (0001–0005)
- Repository foundation in storage/repos.rs
- CLI foundation with local health/config/database commands
- 8 unit/component tests

## [0.1.0] — Initial Scaffold

- Rust workspace with 3 crates (warehouse_core, warehouse_ffi, warehouse_cli)
- Project structure and build configuration
