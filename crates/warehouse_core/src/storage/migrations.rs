use crate::error::{CoreError, CoreResult};
use sqlx::SqlitePool;

/// Represents one migration step.
#[derive(Debug, Clone)]
pub struct Migration {
    pub version: i32,
    pub description: &'static str,
    pub sql: &'static str,
}

/// Returns all migrations in version order.
pub fn all_migrations() -> Vec<Migration> {
    vec![
        Migration {
            version: 1,
            description: "Initial schema: profile_metadata, schema_migrations",
            sql: include_str!("../../../../migrations/sqlite/0001_initial.sql"),
        },
        Migration {
            version: 2,
            description: "Full warehouse client schema: domains, drafts, outbox, sync state",
            sql: include_str!("../../../../migrations/sqlite/0002_full_schema.sql"),
        },
        Migration {
            version: 3,
            description: "Documents cache and report cache tables",
            sql: include_str!("../../../../migrations/sqlite/0003_documents_and_reports.sql"),
        },
        Migration {
            version: 4,
            description: "Outbox enrich: command_type, idempotency_key, payload_hash, server_result",
            sql: include_str!("../../../../migrations/sqlite/0004_outbox_enrich.sql"),
        },
        Migration {
            version: 5,
            description: "Asset operation_id/operation_line_id INTEGER → TEXT (UUID from server)",
            sql: include_str!("../../../../migrations/sqlite/0005_operation_id_text.sql"),
        },
        Migration {
            version: 6,
            description: "Sync run enrichment: families_json and mode columns",
            sql: include_str!("../../../../migrations/sqlite/0006_sync_runs_enrich.sql"),
        },
        Migration {
            version: 7,
            description: "Report cache operation_id INTEGER → TEXT for UUID support",
            sql: include_str!("../../../../migrations/sqlite/0007_report_operation_id_text.sql"),
        },
        Migration {
            version: 8,
            description: "Add catalog audit user fields (created_by_user_id, updated_by_user_id, created_by_user_name, updated_by_user_name)",
            sql: include_str!("../../../../migrations/sqlite/0008_catalog_audit_fields.sql"),
        },
    ]
}

/// Ensure the schema_migrations table exists.
async fn ensure_migrations_table(pool: &SqlitePool) -> CoreResult<()> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            description TEXT NOT NULL,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(pool)
    .await
    .map_err(|e| CoreError::Database(format!("Failed to create schema_migrations: {e}")))?;
    Ok(())
}

/// Get the current schema version (0 if no migrations applied).
async fn current_version(pool: &SqlitePool) -> CoreResult<i32> {
    let result =
        sqlx::query_scalar::<_, i32>("SELECT COALESCE(MAX(version), 0) FROM schema_migrations")
            .fetch_one(pool)
            .await;

    match result {
        Ok(v) => Ok(v),
        Err(sqlx::Error::Database(e)) if e.message().contains("no such table") => Ok(0),
        Err(e) => Err(CoreError::Database(format!(
            "Failed to read migration version: {e}"
        ))),
    }
}

/// Apply all pending migrations in a transaction.
pub async fn run_migrations(pool: &SqlitePool) -> CoreResult<Vec<i32>> {
    ensure_migrations_table(pool).await?;
    let current = current_version(pool).await?;
    let migrations = all_migrations();

    let mut applied = Vec::new();

    for migration in &migrations {
        if migration.version > current {
            let mut tx = pool
                .begin()
                .await
                .map_err(|e| CoreError::Database(format!("Failed to begin transaction: {e}")))?;

            sqlx::query(migration.sql)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    CoreError::Database(format!("Migration v{} failed: {e}", migration.version))
                })?;

            sqlx::query("INSERT INTO schema_migrations (version, description) VALUES (?, ?)")
                .bind(migration.version)
                .bind(migration.description)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    CoreError::Database(format!(
                        "Failed to record migration v{}: {e}",
                        migration.version
                    ))
                })?;

            tx.commit().await.map_err(|e| {
                CoreError::Database(format!(
                    "Failed to commit migration v{}: {e}",
                    migration.version
                ))
            })?;

            applied.push(migration.version);
        }
    }

    Ok(applied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn test_pool() -> SqlitePool {
        SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create test pool")
    }

    #[tokio::test]
    async fn empty_db_returns_version_zero() {
        let pool = test_pool().await;
        let v = current_version(&pool).await.unwrap();
        assert_eq!(v, 0);
    }

    #[tokio::test]
    async fn run_migrations_on_empty_db() {
        let pool = test_pool().await;
        let applied = run_migrations(&pool).await.unwrap();
        assert!(applied.contains(&1));
        assert!(applied.contains(&2));
        assert!(applied.contains(&3));
        assert!(applied.contains(&4));
        assert!(applied.contains(&5));
        assert!(applied.contains(&6));
        assert!(applied.contains(&7));
        assert!(applied.contains(&8));
        assert_eq!(applied.len(), 8);
    }

    #[tokio::test]
    async fn migrations_are_idempotent() {
        let pool = test_pool().await;
        let first = run_migrations(&pool).await.unwrap();
        let second = run_migrations(&pool).await.unwrap();
        assert!(!first.is_empty());
        assert!(second.is_empty());
    }

    #[tokio::test]
    async fn version_after_migration_is_correct() {
        let pool = test_pool().await;
        run_migrations(&pool).await.unwrap();
        let v = current_version(&pool).await.unwrap();
        assert_eq!(v, 8);
    }
}
