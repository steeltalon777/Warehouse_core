//! Local SQLite storage layer.
//!
//! This module contains:
//! - Migration runner
//! - Repository layer for all domain entities
//! - Transaction helper
//! - Database open modes (create, migrate, readonly, reset)
//! - Query builders and indexes

pub mod cursor_store;
pub mod migrations;
pub mod repos;
pub mod snapshot_writer;

use crate::error::CoreResult;
use migrations::run_migrations;
use sqlx::SqlitePool;

/// Build an sqlx-compatible SQLite URL from a file path.
/// Handles Windows absolute paths (e.g. C:\path → sqlite:///C:/path).
pub fn sqlite_url(path: &str) -> String {
    if path == ":memory:" {
        return "sqlite::memory:".into();
    }
    // sqlx passes the string directly to rusqlite, which on Windows needs raw paths.
    // Using `sqlite:` or `file:` URI prefixes can cause issues with absolute paths.
    path.to_string()
}

/// Database handle wrapping the connection pool.
#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
    db_path: String,
}

impl Database {
    /// Open or create the database at the given file path, running migrations.
    /// The file path is converted to an sqlx-compatible URL automatically.
    pub async fn open(db_path: &str, run: bool) -> CoreResult<Self> {
        if db_path != ":memory:" {
            let p = std::path::Path::new(db_path);
            if let Some(parent) = p.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        crate::error::CoreError::Database(format!(
                            "Failed to create DB parent directory: {e}"
                        ))
                    })?;
                }
            }
            // Create DB file if it doesn't exist — SQLite pool connection may not auto-create on Windows
            if !p.exists() {
                std::fs::File::create(db_path).map_err(|e| {
                    crate::error::CoreError::Database(format!("Failed to create DB file: {e}"))
                })?;
            }
        }
        let db_url = sqlite_url(db_path);
        let pool = SqlitePool::connect(&db_url)
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("Failed to connect: {}", e)))?;

        let display_path = if db_path == ":memory:" {
            ":memory:".into()
        } else {
            db_path.to_string()
        };
        let db = Self {
            pool,
            db_path: display_path,
        };

        if run {
            db.migrate().await?;
        }

        Ok(db)
    }

    /// Run pending migrations.
    pub async fn migrate(&self) -> CoreResult<()> {
        let applied = run_migrations(&self.pool).await?;
        if !applied.is_empty() {
            tracing::info!("Applied migrations: {:?}", applied);
        }
        Ok(())
    }

    /// Get the database path.
    pub fn path(&self) -> &str {
        &self.db_path
    }

    /// Get a reference to the connection pool.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn open_in_memory_database() {
        let db = Database::open(":memory:", false).await;
        assert!(db.is_ok());
    }

    #[tokio::test]
    async fn open_and_migrate_in_memory() {
        let db = Database::open(":memory:", true).await.unwrap();
        let v: i32 = sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) FROM schema_migrations")
            .fetch_one(db.pool())
            .await
            .unwrap();
        assert!(v > 0);
    }
}
