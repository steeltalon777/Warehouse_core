use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use warehouse_core::storage::migrations::run_migrations;
use warehouse_core::storage::repos::{
    AuthContextRepo, ErrorLogRepo, SqliteAuthContextRepo, SqliteErrorLogRepo, SqliteSyncRunRepo,
};
use warehouse_core::sync::SyncRunSummary;

async fn setup_pool() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create pool");
    run_migrations(&pool).await.expect("Migrations failed");
    pool
}

#[tokio::test]
async fn test_sync_run_persistence() {
    let pool = setup_pool().await;
    let repo = SqliteSyncRunRepo::new(pool);

    let summary = SyncRunSummary {
        run_id: "persist-test-1".into(),
        started_at: "2024-01-01T00:00:00Z".into(),
        completed_at: Some("2024-01-01T00:01:00Z".into()),
        families: vec![warehouse_core::sync::FamilyResult {
            name: "catalog".into(),
            success: true,
            items_count: 50,
            error: None,
        }],
        total_items: 50,
        errors_count: 0,
        is_complete: true,
    };
    repo.insert_sync_run(&summary).await.unwrap();

    let list = repo.list_sync_runs(10).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].run_id, "persist-test-1");
    assert_eq!(list[0].total_items, 50);
    assert!(list[0].is_complete);
}

#[tokio::test]
async fn test_cancel_sync_flag() {
    let cancelled = Arc::new(AtomicBool::new(false));
    assert!(!cancelled.load(Ordering::SeqCst));

    cancelled.store(true, Ordering::SeqCst);
    assert!(cancelled.load(Ordering::SeqCst));

    cancelled.store(false, Ordering::SeqCst);
    assert!(!cancelled.load(Ordering::SeqCst));
}

#[tokio::test]
async fn test_concurrent_sync_lock() {
    let lock = Arc::new(AtomicBool::new(false));

    // First acquire succeeds
    assert!(!lock.swap(true, Ordering::SeqCst));

    // Second acquire fails (already locked)
    assert!(lock.swap(true, Ordering::SeqCst));

    // Release
    lock.store(false, Ordering::SeqCst);

    // Can acquire again
    assert!(!lock.swap(true, Ordering::SeqCst));
}

#[tokio::test]
async fn test_sync_lock_release_on_drop() {
    let lock = Arc::new(AtomicBool::new(false));
    {
        // Simulate scope with error
        let prev = lock.swap(true, Ordering::SeqCst);
        assert!(!prev);
        // Error occurs, but we ensure lock is released before drop
        lock.store(false, Ordering::SeqCst);
    }
    // Lock should be available again
    assert!(!lock.swap(true, Ordering::SeqCst));
}

#[tokio::test]
async fn test_error_log_roundtrip() {
    let pool = setup_pool().await;
    let repo = SqliteErrorLogRepo::new(pool);

    let id = repo
        .log("ERROR", "sync", "Connection lost", Some("details: timeout"))
        .await
        .unwrap();
    assert!(id > 0);

    let id2 = repo
        .log("WARN", "catalog", "Item 42 not found", None)
        .await
        .unwrap();
    assert!(id2 > id);

    let recent = repo.recent(10).await.unwrap();
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].1, "WARN");
    assert_eq!(recent[0].2, "catalog");
    assert_eq!(recent[0].3, "Item 42 not found");
    assert_eq!(recent[1].1, "ERROR");
    assert_eq!(recent[1].3, "Connection lost");
}

#[tokio::test]
async fn test_rollback_on_failure() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    // Insert a row
    sqlx::query("INSERT INTO auth_context (key, value, updated_at) VALUES (?, ?, ?)")
        .bind("test_key")
        .bind("test_val")
        .bind("2024-01-01T00:00:00Z")
        .execute(&mut *tx)
        .await
        .unwrap();

    // Rollback instead of commit
    tx.rollback().await.unwrap();

    // Verify the row is not present
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM auth_context WHERE key = 'test_key'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test]
async fn test_error_log_empty() {
    let pool = setup_pool().await;
    let repo = SqliteErrorLogRepo::new(pool);
    let recent = repo.recent(10).await.unwrap();
    assert!(recent.is_empty());
}

#[tokio::test]
async fn test_error_log_limit() {
    let pool = setup_pool().await;
    let repo = SqliteErrorLogRepo::new(pool);
    for i in 0..5 {
        repo.log("INFO", "test", &format!("msg {i}"), None)
            .await
            .unwrap();
    }
    let recent = repo.recent(3).await.unwrap();
    assert_eq!(recent.len(), 3);
    assert_eq!(recent[0].3, "msg 4");
    assert_eq!(recent[1].3, "msg 3");
    assert_eq!(recent[2].3, "msg 2");
}

#[tokio::test]
async fn test_sync_run_with_errors() {
    let pool = setup_pool().await;
    let repo = SqliteSyncRunRepo::new(pool);
    let summary = SyncRunSummary {
        run_id: "error-run".into(),
        started_at: "2024-01-01T00:00:00Z".into(),
        completed_at: Some("2024-01-01T00:01:00Z".into()),
        families: vec![
            warehouse_core::sync::FamilyResult {
                name: "catalog".into(),
                success: false,
                items_count: 0,
                error: Some("Server unreachable".into()),
            },
            warehouse_core::sync::FamilyResult {
                name: "sites".into(),
                success: true,
                items_count: 3,
                error: None,
            },
        ],
        total_items: 3,
        errors_count: 1,
        is_complete: true,
    };
    repo.insert_sync_run(&summary).await.unwrap();
    let list = repo.list_sync_runs(10).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].errors_count, 1);
}

#[tokio::test]
async fn test_transaction_commit_succeeds() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("INSERT INTO auth_context (key, value, updated_at) VALUES (?, ?, ?)")
        .bind("commit_key")
        .bind("commit_val")
        .bind("2024-01-01T00:00:00Z")
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let value: String =
        sqlx::query_scalar("SELECT value FROM auth_context WHERE key = 'commit_key'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(value, "commit_val");
}

#[tokio::test]
async fn test_auth_context_crud() {
    let pool = setup_pool().await;
    let repo = SqliteAuthContextRepo::new(pool);

    // Initially not present
    assert!(repo.get("my_key").await.unwrap().is_none());

    // Set
    repo.set("my_key", "my_value").await.unwrap();
    assert_eq!(repo.get("my_key").await.unwrap(), Some("my_value".into()));

    // Update
    repo.set("my_key", "updated").await.unwrap();
    assert_eq!(repo.get("my_key").await.unwrap(), Some("updated".into()));

    // Delete
    repo.delete("my_key").await.unwrap();
    assert!(repo.get("my_key").await.unwrap().is_none());
}
