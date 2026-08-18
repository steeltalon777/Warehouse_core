# Warehouse Client Core

Этот компонент Quartermaster: offline-first Rust runtime.

Offline-first Rust runtime for Warehouse Desktop (WPF) and Warehouse Mobile (Android) clients.

## Project Status

**Levels 0–10 complete, Levels 11–12 hardening underway.**

- Levels 0–6: SyncServer HTTP client (13 modules), Auth/Profile/Bootstrap, SQLite repos + migrations (6), Pull Sync (12 families), Read-only CoreFacade + CLI, FFI foundation (48 extern "C" functions)
- Level 7: OperationDraftService — offline draft CRUD, validation, to_operation_create conversion
- Level 8: OutboxService — durable command queue, idempotency, retry with backoff, conflict/dead-letter classification
- Level 9: SyncEngine — 5-sync-mode orchestrator with progress callback, lock, conflict collection
- Level 10: Full facade through FFI — 92 CoreHandle methods, 48 extern "C" exports
- Level 11 (in progress): Contract, performance, chaos, regression hardening
- Level 12 (in progress): Documentation, packaging, client handoff

## Crate Structure

| Crate | Type | Key Modules |
|---|---|---|
| `warehouse_core` | library | `domain/` (DTOs), `storage/` (SQLite + repos + migrations), `syncserver/` (HTTP client), `auth/` (identity), `sync/` (PullSync, Bootstrap, SyncEngine), `operations/` (drafts, outbox), `facade/` (CoreHandle) |
| `warehouse_ffi` | cdylib | 48 `extern "C"` exports, `CoreErrorDto` repr(C), `FfiTokenProvider` |
| `warehouse_cli` | binary | 14 command groups, 30+ subcommands |

## Prerequisites

- Rust stable ≥ 1.75 (tested with 1.85+)
- SQLite3 (included via `rusqlite` with bundled feature)
- `tokio` runtime (async)
- Windows: Visual Studio Build Tools or MSVC toolchain
- Optional: Android NDK for `arm64-v8a` / `x86_64` targets

## Build

```bash
cargo build --workspace
cargo build -p warehouse_ffi          # produces target/debug/libwarehouse_ffi.so / .dll / .dylib
cargo build -p warehouse_cli           # produces target/debug/warehouse-cli
```

## Static Checks

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

## Test

```bash
cargo test --workspace                 # 35 tests (SQLite repos + HTTP client + facade + FFI)
```

## CLI Usage

```bash
# Health
cargo run -p warehouse_cli -- health-local
cargo run -p warehouse_cli -- health-remote --server http://localhost:8000

# Auth
cargo run -p warehouse_cli -- auth-context
cargo run -p warehouse_cli -- site list
cargo run -p warehouse_cli -- site set --site-id 1

# Sync
cargo run -p warehouse_cli -- bootstrap --server http://localhost:8000
cargo run -p warehouse_cli -- sync-pull
cargo run -p warehouse_cli -- sync full

# Catalog
cargo run -p warehouse_cli -- catalog search --query bolt
cargo run -p warehouse_cli -- balances list --site-id 1

# Operations
cargo run -p warehouse_cli -- draft create --type RECEIVE
cargo run -p warehouse_cli -- draft list
cargo run -p warehouse_cli -- draft submit --draft-id <uuid>
cargo run -p warehouse_cli -- operations list --site-id 1

# Outbox
cargo run -p warehouse_cli -- outbox list
cargo run -p warehouse_cli -- outbox send

# Temp items / Assets
cargo run -p warehouse_cli -- temp-items list
cargo run -p warehouse_cli -- assets pending
cargo run -p warehouse_cli -- assets lost
cargo run -p warehouse_cli -- assets issued

# Issue Objects
cargo run -p warehouse_cli -- issue-objects list
cargo run -p warehouse_cli -- issue-objects categories

# Documents / Reports
cargo run -p warehouse_cli -- documents list --site-id 1
cargo run -p warehouse_cli -- reports stock-summary
```

## Client Integration

| Client | Binding | Doc |
|---|---|---|
| Android (Kotlin) | UniFFI or C ABI via JNI | `docs/ANDROID_BINDINGS.md` |
| WPF (C#) | P/Invoke on `warehouse_ffi` | `docs/WPF_BINDINGS.md` |
| Real test stand | SyncServer + PostgreSQL | `docs/SYNC_STAND.md` |

## Documentation

| File | Content |
|---|---|
| `docs/CORE_FACADE_V1.md` | Complete method/DTO/error reference (92 methods) |
| `docs/ANDROID_BINDINGS.md` | Android integration instructions |
| `docs/WPF_BINDINGS.md` | WPF integration instructions |
| `docs/SYNC_STAND.md` | Real stand setup and smoke tests |
| `docs/TZ_CORE_CLIENT_READY_COMPLETION.md` | Implementation roadmap (13 levels) |
| `docs/RUST_CORE_CAPABILITY_PLAN.md` | Capability levels |
| `docs/CORE_STAND_SMOKE_REPORT.md` | Latest stand smoke test report |
| `CHANGELOG.md` | Release history |

## License

MIT
