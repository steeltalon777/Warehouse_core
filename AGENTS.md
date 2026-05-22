# Warehouse_client_core Agent Contract

## Role

`Warehouse_client_core` is the planned Rust offline-first runtime for future desktop and mobile clients.

## Functional Requirements Authority

- `Functional and WorkLogik.md` at the workspace root is the **canonical functional requirements document** for operation types, user roles, lifecycle rules, and domain behaviour.
- Before implementing any sync logic, operation draft service, or domain behaviour, re-read the relevant section of `Functional and WorkLogik.md` and confirm alignment.

## Rules

- SyncServer remains the source of truth.
- The core may cache server data and local user intent, but it must not become an authoritative backend.
- Put local SQLite schema, outbox, sync, DTO mapping, conflict state, and a stable facade API here.
- Keep WPF, Android, and AI workstation UI logic out of this project.
- Do not start production implementation before architecture decisions are captured in docs/ADR.

## Git Rules

- Parallel sessions are normal. `git status` may show unrelated modified/untracked files from other agents or the user; this is not a blocker by itself.
- Agents MUST commit their own completed Rust core changes after relevant checks/tests pass.
- Commit only from the `dev` branch.
- Switching from `dev` to another branch is forbidden by default.
- If the branch is not `dev`, warn the user and do not commit until the user gives an explicit command.
- Stage and commit only files intentionally changed for the assigned task, using explicit pathspecs such as `git add -- path/to/file`. Do not use broad `git add .` or `git add -A` for task commits.
- Git does not auto-track new files by itself; untracked files become tracked only after staging. Keep local/service artifacts ignored and unstaged unless explicitly assigned.
- Before committing, inspect the staged diff and confirm it contains only task-owned files. Leave unrelated dirty files unstaged.
- If intended edits overlap with unrelated changes in the same file, stop and report the ownership conflict instead of committing.
- If checks/tests fail, are unavailable, or were not run, do not commit unless the user explicitly instructs to commit with that limitation documented.
- Git push is completely forbidden; the user pushes manually.

## Active TZ

- Use `docs/TZ_CORE_CLIENT_READY_COMPLETION.md` as the executable roadmap for making the Rust core client-ready.
- Use `docs/RUST_CORE_CAPABILITY_PLAN.md` as the broader capability map.

## Verification

```bash
# Static checks
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

# All tests (8 SQLite + 21 contract/HTTP = 29 total)
cargo test --workspace

# Build all 3 crates
cargo build --workspace

# FFI cdylib build
cargo build -p warehouse_ffi
```

## Documentation

| File | Content |
|---|---|
| `docs/CORE_FACADE_V1.md` | Complete method reference (70+ methods) |
| `docs/ANDROID_BINDINGS.md` | Android integration with `warehouse_ffi` |
| `docs/WPF_BINDINGS.md` | WPF integration with `warehouse_ffi` |
| `docs/SYNC_STAND.md` | Real SyncServer test stand setup |
| `docs/TZ_CORE_CLIENT_READY_COMPLETION.md` | Implementation roadmap |

## Crate Structure

| Crate | Type | Key Modules |
|---|---|---|
| `warehouse_core` | library | `domain/` (DTOs), `storage/` (SQLite + repos), `syncserver/` (HTTP client), `auth/` (identity), `sync/` (engine), `operations/` (drafts, outbox), `facade/` (CoreHandle) |
| `warehouse_ffi` | cdylib | 48 `extern "C"` exports, `CoreErrorDto` repr(C) |
| `warehouse_cli` | binary | 11 command groups, 30+ subcommands |
