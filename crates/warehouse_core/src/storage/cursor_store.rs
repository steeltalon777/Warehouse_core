use crate::error::CoreResult;
use crate::storage::repos::{SqliteSyncCursorRepo, SyncCursorRepo};
use sqlx::SqlitePool;

/// Cursor keys for each pull family
pub mod keys {
    pub const CATALOG_ITEMS: &str = "catalog_items";
    pub const CATALOG_CATEGORIES: &str = "catalog_categories";
    pub const CATALOG_UNITS: &str = "catalog_units";
    pub const CATEGORY_TREE: &str = "category_tree";
    pub const SITES: &str = "sites";
    pub const RECIPIENTS: &str = "recipients";
    pub const TEMPORARY_ITEMS: &str = "temporary_items";
    pub const OPERATIONS: &str = "operations";
    pub const DOCUMENTS: &str = "documents";
    pub const PENDING_ACCEPTANCE: &str = "pending_acceptance";
    pub const LOST_ASSETS: &str = "lost_assets";
    pub const ISSUED_ASSETS: &str = "issued_assets";
    pub const STOCK_SUMMARY: &str = "stock_summary";
    pub const SERVER_SEQ: &str = "server_seq";
    pub fn balances(site_id: i32) -> String {
        format!("balances_site_{site_id}")
    }
}

/// Status info stored alongside cursor value
#[derive(Debug, Clone)]
pub struct CursorInfo {
    pub cursor_value: Option<String>,
    pub server_time: Option<String>,
    pub refreshed_at: String,
    pub error: Option<String>,
}

/// CursorStore: get/set cursor state per pull family.
pub struct CursorStore {
    repo: SqliteSyncCursorRepo,
}

impl CursorStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            repo: SqliteSyncCursorRepo::new(pool),
        }
    }

    pub async fn get_updated_after(&self, key: &str) -> CoreResult<Option<String>> {
        self.repo.get_cursor(key).await
    }

    pub async fn set_updated_after(&self, key: &str, value: &str) -> CoreResult<()> {
        self.repo.set_cursor(key, value).await
    }

    pub async fn get_server_seq(&self) -> CoreResult<Option<i64>> {
        let val = self.repo.get_cursor(keys::SERVER_SEQ).await?;
        match val {
            Some(s) => s.parse::<i64>().map(Some).map_err(|e| {
                crate::error::CoreError::Database(format!("Invalid server_seq cursor: {e}"))
            }),
            None => Ok(None),
        }
    }

    pub async fn set_server_seq(&self, seq: i64) -> CoreResult<()> {
        self.repo
            .set_cursor(keys::SERVER_SEQ, &seq.to_string())
            .await
    }

    pub async fn clear(&self, key: &str) -> CoreResult<()> {
        let pool = self.repo.pool.clone();
        sqlx::query("DELETE FROM sync_cursors WHERE cursor_type = ?")
            .bind(key)
            .execute(&pool)
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        Ok(())
    }

    pub async fn all_cursors(&self) -> CoreResult<Vec<(String, String)>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            cursor_type: String,
            cursor_value: String,
        }
        let rows: Vec<Row> = sqlx::query_as(
            "SELECT cursor_type, cursor_value FROM sync_cursors ORDER BY cursor_type",
        )
        .fetch_all(&self.repo.pool)
        .await
        .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        Ok(rows
            .into_iter()
            .map(|r| (r.cursor_type, r.cursor_value))
            .collect())
    }
}
