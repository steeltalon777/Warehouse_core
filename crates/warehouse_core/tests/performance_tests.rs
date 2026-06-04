use std::time::Instant;
use warehouse_core::domain::catalog::{CategoryDto, UnitDto};
use warehouse_core::storage::migrations::run_migrations;
use warehouse_core::storage::repos::{
    CatalogRepo, DraftRepo, OutboxRepo, SqliteCatalogRepo, SqliteDraftRepo, SqliteOutboxRepo,
};

async fn setup_pool() -> sqlx::SqlitePool {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create pool");
    run_migrations(&pool).await.expect("Migrations failed");
    pool
}

#[ignore]
#[tokio::test]
async fn bench_catalog_10k_items() {
    let pool = setup_pool().await;
    let repo = SqliteCatalogRepo::new(pool.clone());

    let unit = UnitDto {
        id: 1,
        name: "pcs".into(),
        symbol: "pcs".into(),
        is_active: true,
        updated_at: "2024-01-01T00:00:00Z".into(),
        created_by_user_id: None,
        updated_by_user_id: None,
        created_by_user_name: None,
        updated_by_user_name: None,
    };
    let cat = CategoryDto {
        id: 1,
        name: "Bench".into(),
        parent_id: None,
        is_active: true,
        updated_at: "2024-01-01T00:00:00Z".into(),
        created_by_user_id: None,
        updated_by_user_id: None,
        created_by_user_name: None,
        updated_by_user_name: None,
    };
    repo.upsert_unit(&unit).await.unwrap();
    repo.upsert_category(&cat).await.unwrap();

    let start = Instant::now();
    for i in 1..=10_000 {
        let item = warehouse_core::domain::catalog::ItemDto {
            id: i,
            sku: Some(format!("BENCH-{i:05}")),
            name: format!("Performance test item {i}"),
            category_id: 1,
            unit_id: 1,
            description: None,
            is_active: true,
            hashtags: None,
            updated_at: "2024-01-01T00:00:00Z".into(),
            created_at: Some("2024-01-01T00:00:00Z".into()),
            created_by_user_id: None,
            updated_by_user_id: None,
            created_by_user_name: None,
            updated_by_user_name: None,
        };
        repo.upsert_item(&item).await.unwrap();
    }
    let insert_dur = start.elapsed();
    eprintln!("Insert 10k items: {:?}", insert_dur);

    let search_start = Instant::now();
    let results = repo.search_items("Performance").await.unwrap();
    let search_dur = search_start.elapsed();
    eprintln!(
        "Search 10k items: {:?} (found {})",
        search_dur,
        results.len()
    );

    assert_eq!(results.len(), 10_000);
}

#[ignore]
#[tokio::test]
async fn bench_large_category_tree() {
    let pool = setup_pool().await;
    let repo = SqliteCatalogRepo::new(pool);

    let start = Instant::now();
    // Insert root category
    let root = CategoryDto {
        id: 1,
        name: "Root".into(),
        parent_id: None,
        is_active: true,
        updated_at: "2024-01-01T00:00:00Z".into(),
        created_by_user_id: None,
        updated_by_user_id: None,
        created_by_user_name: None,
        updated_by_user_name: None,
    };
    repo.upsert_category(&root).await.unwrap();

    // Insert 999 children
    for i in 2..=1000 {
        let cat = CategoryDto {
            id: i,
            name: format!("Category {i}"),
            parent_id: Some(i / 2),
            is_active: true,
            updated_at: "2024-01-01T00:00:00Z".into(),
            created_by_user_id: None,
            updated_by_user_id: None,
            created_by_user_name: None,
            updated_by_user_name: None,
        };
        repo.upsert_category(&cat).await.unwrap();
    }
    let dur = start.elapsed();
    eprintln!("Insert 1k category tree: {:?}", dur);

    let all = repo.all_categories().await.unwrap();
    assert_eq!(all.len(), 1000);
}

#[ignore]
#[tokio::test]
async fn bench_1k_outbox_entries() {
    let pool = setup_pool().await;
    let repo = SqliteOutboxRepo::new(pool);

    let start = Instant::now();
    for _ in 0..1000 {
        repo.enqueue("bench", "cmd", 1, "{}", None, None)
            .await
            .unwrap();
    }
    let insert_dur = start.elapsed();
    eprintln!("Insert 1k outbox entries: {:?}", insert_dur);

    let list_start = Instant::now();
    let list = repo.list(None, None).await.unwrap();
    let list_dur = list_start.elapsed();
    eprintln!("List 1k outbox: {:?} (count={})", list_dur, list.len());

    let count_start = Instant::now();
    let count = repo.count_pending().await.unwrap();
    let count_dur = count_start.elapsed();
    eprintln!("Count pending 1k outbox: {:?} (count={})", count_dur, count);

    assert_eq!(list.len(), 1000);
    assert_eq!(count, 1000);
}

#[ignore]
#[tokio::test]
async fn bench_draft_with_100_lines() {
    let pool = setup_pool().await;
    let repo = SqliteDraftRepo::new(pool);

    let start = Instant::now();
    let draft = warehouse_core::domain::operation::OperationDraft {
        draft_id: uuid::Uuid::new_v4(),
        operation_type: warehouse_core::domain::operation::OperationType::Receive,
        site_id: Some(1),
        lines: (0..100)
            .map(|i| warehouse_core::domain::operation::OperationDraftLine {
                line_id: uuid::Uuid::new_v4(),
                item_id: Some(i + 1),
                temporary_item: None,
                qty: serde_json::json!("1"),
                batch: None,
                comment: if i % 2 == 0 {
                    Some(format!("Line {i}"))
                } else {
                    None
                },
            })
            .collect(),
        effective_at: None,
        source_site_id: None,
        destination_site_id: None,
        recipient_id: None,
        issued_to_name: None,
        comment: None,
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
    };
    repo.save(&draft).await.unwrap();
    let save_dur = start.elapsed();
    eprintln!("Save draft with 100 lines: {:?}", save_dur);

    let get_start = Instant::now();
    let got = repo
        .get(&draft.draft_id.to_string())
        .await
        .unwrap()
        .expect("Draft not found");
    let get_dur = get_start.elapsed();
    eprintln!("Get draft with 100 lines: {:?}", get_dur);

    assert_eq!(got.lines.len(), 100);
}

#[ignore]
#[tokio::test]
async fn bench_migration_large_profile() {
    let start = Instant::now();
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("Failed to create pool");
    run_migrations(&pool).await.expect("Migrations failed");
    let dur = start.elapsed();
    eprintln!("Run all migrations: {:?}", dur);
    // Verify version
    let version =
        sqlx::query_scalar::<_, i32>("SELECT COALESCE(MAX(version), 0) FROM schema_migrations")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(version, 6);
}
