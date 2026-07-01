//! Stand smoke test for Rust core ↔ SyncServer compatibility.
//!
//! Runs against the real dev SyncServer. Skips gracefully when required env
//! vars are not set. Marked `#[ignore]` so it does not run on plain
//! `cargo test --workspace`; invoke with `cargo test -- --ignored`.
//!
//! Required env vars:
//! - `SYNC_SERVER_URL`        (e.g. `http://localhost:8000`)
//! - `SYNC_USER_TOKEN`        (Uuid of the root user token)
//! - `SYNC_DEVICE_TOKEN`      (Uuid of the registered device token)
//!
//! Scenario: bootstrap → pull catalog → create a draft → queue outbox →
//! push → pull. After completion the local catalog must contain the rows
//! loaded from the server.

use std::path::PathBuf;
use warehouse_core::auth::token_provider::FfiTokenProvider;
use warehouse_core::config::CoreConfig;
use warehouse_core::domain::operation::OperationType;
use warehouse_core::facade::CoreHandle;
use warehouse_core::sync::SyncMode;

fn read_env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|s| !s.is_empty())
}

fn skip_if_missing_tokens() -> Option<(String, uuid::Uuid, uuid::Uuid)> {
    // The dev stand is reached on the host as `http://localhost:8000`. The
    // workspace `.env` carries an internal Docker URL (`http://syncserver:8000`),
    // which is not resolvable from the host where `cargo test` runs. Default to
    // localhost and only honour an explicit host-style override via
    // `WAREHOUSE_STAND_URL`.
    let server_url =
        read_env("WAREHOUSE_STAND_URL").or_else(|| Some("http://localhost:8000".to_string()))?;
    let user_token = read_env("SYNC_USER_TOKEN")
        .or_else(|| read_env("WAREHOUSE_USER_TOKEN"))
        .or_else(|| read_env("SYNC_ROOT_USER_TOKEN"))?;
    let device_token =
        read_env("SYNC_DEVICE_TOKEN").or_else(|| read_env("WAREHOUSE_DEVICE_TOKEN"))?;
    let user = uuid::Uuid::parse_str(&user_token)
        .map_err(|e| eprintln!("[stand_smoke] invalid user token: {e}"))
        .ok()?;
    let device = uuid::Uuid::parse_str(&device_token)
        .map_err(|e| eprintln!("[stand_smoke] invalid device token: {e}"))
        .ok()?;
    Some((server_url, user, device))
}

fn unique_tmp_db(label: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("warehouse_stand_smoke_{label}_{pid}_{nanos}.db"))
}

#[tokio::test]
#[ignore = "requires real SyncServer and tokens; run with --ignored"]
async fn stand_smoke_bootstrap_push_pull() {
    let (server_url, user_token, device_token) = match skip_if_missing_tokens() {
        Some(v) => v,
        None => {
            eprintln!(
                "[stand_smoke] SKIPPED: SYNC_SERVER_URL / SYNC_USER_TOKEN / SYNC_DEVICE_TOKEN not set"
            );
            return;
        }
    };

    eprintln!("[stand_smoke] server={server_url}");

    let db_path = unique_tmp_db("smoke");
    let _ = std::fs::remove_file(&db_path);

    let config = CoreConfig {
        server_base_url: server_url.clone(),
        database_path: db_path.clone(),
        ..CoreConfig::for_testing(db_path.clone())
    };

    let mut handle = CoreHandle::open(config)
        .await
        .expect("CoreHandle::open should succeed against local SQLite");

    let tokens = FfiTokenProvider::new();
    tokens.set_user_token(user_token);
    tokens.set_device_token(device_token);
    handle.set_token_provider(Box::new(tokens));

    // 1. Bootstrap
    let bootstrap = handle
        .bootstrap()
        .await
        .expect("bootstrap should succeed against dev stand");
    assert!(
        bootstrap.success,
        "bootstrap reported failure: {:?}",
        bootstrap.errors
    );
    eprintln!(
        "[stand_smoke] bootstrap ok, families={:?}, protocol={}",
        bootstrap.families_synced, bootstrap.protocol_version
    );

    // 2. Catalog must be loaded.
    let categories = handle
        .list_categories()
        .await
        .expect("list_categories after bootstrap");
    let units = handle
        .list_units()
        .await
        .expect("list_units after bootstrap");
    let items = handle
        .search_items("")
        .await
        .expect("search_items after bootstrap");
    eprintln!(
        "[stand_smoke] catalog: categories={}, units={}, items={}",
        categories.len(),
        units.len(),
        items.len()
    );
    assert!(!categories.is_empty(), "categories not loaded by bootstrap");
    assert!(!units.is_empty(), "units not loaded by bootstrap");
    assert!(!items.is_empty(), "items not loaded by bootstrap");

    // 3. Pick a site from the auth context and create a draft.
    let ctx = handle
        .get_auth_context()
        .expect("auth context after bootstrap");
    let site_id = ctx
        .available_sites
        .first()
        .map(|s| s.site_id)
        .expect("at least one site available after bootstrap");

    let draft = handle
        .create_draft(OperationType::Receive, Some(site_id))
        .await
        .expect("create_draft should succeed");
    let draft_id = draft.draft_id.to_string();
    eprintln!("[stand_smoke] created draft {draft_id} for site {site_id}");

    let updated = handle
        .add_draft_item_line(&draft_id, items[0].id, serde_json::json!("3"), None, None)
        .await
        .expect("add_draft_item_line")
        .expect("draft must exist after line add");
    eprintln!("[stand_smoke] draft has {} lines", updated.lines.len());

    // 4. Queue the draft for submission (this enqueues an outbox event with
    //    a SHA-256 payload_hash that must match the SyncServer formula).
    let event_uuid = handle
        .queue_draft_submit(&draft_id)
        .await
        .expect("queue_draft_submit should succeed");
    eprintln!("[stand_smoke] enqueued outbox event {event_uuid}");

    // 5. Push the outbox via the sync engine.
    //
    // We assert that the engine dequeued and processed the event — the
    // exact server outcome depends on stand data (accepted, failed, or
    // conflict) and is not part of the gate-5 contract. The important
    // invariant is that the request reached the server and the outbox
    // event was marked with a terminal status.
    let push_result = handle.sync_once(SyncMode::PushOnly).await;
    eprintln!(
        "[stand_smoke] push: accepted={} failed={} conflicts={} error={:?}",
        push_result.push_accepted,
        push_result.push_failed,
        push_result.push_conflicts,
        push_result.error
    );
    let push_processed =
        push_result.push_accepted + push_result.push_failed + push_result.push_conflicts;
    assert!(
        push_processed >= 1,
        "push did not dequeue any outbox events"
    );

    // 6. Pull once to refresh.
    let pull_result = handle.sync_once(SyncMode::PullOnly).await;
    eprintln!(
        "[stand_smoke] pull: items={} errors={} success={} error={:?}",
        pull_result.pull_items, pull_result.pull_errors, pull_result.success, pull_result.error
    );

    // 7. Cleanup.
    drop(handle);
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(format!("{}-wal", db_path.display()));
    let _ = std::fs::remove_file(format!("{}-shm", db_path.display()));
}

#[tokio::test]
#[ignore = "requires real SyncServer and tokens; run with --ignored"]
async fn stand_smoke_payload_hash_matches_syncserver() {
    // Read tokens so we can confirm reachability, but the actual hash check
    // is local (it must match the SyncServer formula regardless of stand).
    if skip_if_missing_tokens().is_none() {
        eprintln!("[stand_smoke] SKIPPED: tokens not set");
        return;
    }
    use warehouse_core::operations::outbox_service::compute_payload_hash;

    // The SyncServer `payload_hash` formula is:
    //   json.dumps(data, ensure_ascii=False, sort_keys=True, default=str)
    //   hashlib.sha256(...).hexdigest()
    //
    // For ASCII-only payloads the Rust and Python serializations are
    // byte-identical, so the hex digest must match exactly.
    //
    // Expected digest (computed with Python 3):
    //   >>> import json, hashlib
    //   >>> data = {"a": 1, "b": [1, 2, 3]}
    //   >>> payload = json.dumps(data, sort_keys=True, separators=(",", ":"))
    //   >>> hashlib.sha256(payload.encode("utf-8")).hexdigest()
    //   'bfa6ceebf136e4837ec687f2be09f612c645c9ec1f99e3ef5d497b0d5bb99e0a'
    let expected = "bfa6ceebf136e4837ec687f2be09f612c645c9ec1f99e3ef5d497b0d5bb99e0a";
    let payload = r#"{"a": 1, "b": [1, 2, 3]}"#;
    let actual = compute_payload_hash(payload);
    assert_eq!(
        actual, expected,
        "Rust SHA-256 must match SyncServer formula"
    );
}
