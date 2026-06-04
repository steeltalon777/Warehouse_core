use crate::error::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

// ── Helpers ─────────────────────────────────────────────────────

macro_rules! map_err {
    ($e:expr) => {
        CoreError::Database(format!("{}", $e))
    };
}

fn uuid_str() -> String {
    Uuid::new_v4().to_string()
}

fn now_str() -> String {
    crate::time::Timestamp::now_utc().to_string()
}

// ── AuthContextRepo ─────────────────────────────────────────────

pub trait AuthContextRepo {
    async fn get(&self, key: &str) -> CoreResult<Option<String>>;
    async fn set(&self, key: &str, value: &str) -> CoreResult<()>;
    async fn delete(&self, key: &str) -> CoreResult<()>;
}

#[derive(Debug, Clone)]
pub struct SqliteAuthContextRepo {
    pool: SqlitePool,
}

impl SqliteAuthContextRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl AuthContextRepo for SqliteAuthContextRepo {
    async fn get(&self, key: &str) -> CoreResult<Option<String>> {
        sqlx::query_scalar::<_, String>("SELECT value FROM auth_context WHERE key = ?")
            .bind(key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| map_err!(e))
    }

    async fn set(&self, key: &str, value: &str) -> CoreResult<()> {
        sqlx::query(
            "INSERT INTO auth_context (key, value, updated_at) VALUES (?, ?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        )
        .bind(key)
        .bind(value)
        .bind(now_str())
        .execute(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> CoreResult<()> {
        sqlx::query("DELETE FROM auth_context WHERE key = ?")
            .bind(key)
            .execute(&self.pool)
            .await
            .map_err(|e| map_err!(e))?;
        Ok(())
    }
}

// ── CatalogRepo ─────────────────────────────────────────────────

pub trait CatalogRepo {
    async fn upsert_item(&self, item: &crate::domain::catalog::ItemDto) -> CoreResult<()>;
    async fn upsert_category(&self, cat: &crate::domain::catalog::CategoryDto) -> CoreResult<()>;
    async fn upsert_unit(&self, unit: &crate::domain::catalog::UnitDto) -> CoreResult<()>;
    async fn get_item(&self, id: i32) -> CoreResult<Option<crate::domain::catalog::ItemDto>>;
    async fn search_items(&self, query: &str) -> CoreResult<Vec<crate::domain::catalog::ItemDto>>;
    async fn all_categories(&self) -> CoreResult<Vec<crate::domain::catalog::CategoryDto>>;
    async fn all_units(&self) -> CoreResult<Vec<crate::domain::catalog::UnitDto>>;
    async fn clear_items(&self) -> CoreResult<()>;
    async fn clear_categories(&self) -> CoreResult<()>;
    async fn clear_units(&self) -> CoreResult<()>;
}

#[derive(Debug, Clone)]
pub struct SqliteCatalogRepo {
    pool: SqlitePool,
}

impl SqliteCatalogRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl CatalogRepo for SqliteCatalogRepo {
    async fn upsert_item(&self, item: &crate::domain::catalog::ItemDto) -> CoreResult<()> {
        let hashtags = item
            .hashtags
            .as_ref()
            .map(|h| serde_json::to_string(h).unwrap_or_default());
        let fallback_time = crate::time::Timestamp::now_utc().to_string();
        let created_at = item.created_at.as_deref().unwrap_or(&fallback_time);
        sqlx::query(
            "INSERT INTO items (id, sku, name, category_id, unit_id, description, is_active, hashtags, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               sku = excluded.sku, name = excluded.name, category_id = excluded.category_id,
               unit_id = excluded.unit_id, description = excluded.description,
               is_active = excluded.is_active, hashtags = excluded.hashtags,
               updated_at = excluded.updated_at",
        )
        .bind(item.id).bind(&item.sku).bind(&item.name)
        .bind(item.category_id).bind(item.unit_id).bind(&item.description)
        .bind(item.is_active).bind(&hashtags).bind(created_at).bind(&item.updated_at)
        .execute(&self.pool).await.map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn upsert_category(&self, cat: &crate::domain::catalog::CategoryDto) -> CoreResult<()> {
        let now = crate::time::Timestamp::now_utc().to_string();
        sqlx::query(
            "INSERT INTO categories (id, name, parent_id, is_active, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               name = excluded.name, parent_id = excluded.parent_id,
               is_active = excluded.is_active, updated_at = excluded.updated_at",
        )
        .bind(cat.id)
        .bind(&cat.name)
        .bind(cat.parent_id)
        .bind(cat.is_active)
        .bind(&now)
        .bind(&cat.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn upsert_unit(&self, unit: &crate::domain::catalog::UnitDto) -> CoreResult<()> {
        let now = crate::time::Timestamp::now_utc().to_string();
        sqlx::query(
            "INSERT INTO units (id, name, symbol, is_active, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               name = excluded.name, symbol = excluded.symbol,
               is_active = excluded.is_active, updated_at = excluded.updated_at",
        )
        .bind(unit.id)
        .bind(&unit.name)
        .bind(&unit.symbol)
        .bind(unit.is_active)
        .bind(&now)
        .bind(&unit.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn get_item(&self, id: i32) -> CoreResult<Option<crate::domain::catalog::ItemDto>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: i32,
            sku: Option<String>,
            name: String,
            category_id: i32,
            unit_id: i32,
            description: Option<String>,
            is_active: bool,
            hashtags: Option<String>,
            updated_at: String,
        }
        let row: Option<Row> = sqlx::query_as("SELECT * FROM items WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| map_err!(e))?;
        Ok(row.map(|r| {
            let hashtags = r.hashtags.and_then(|h| serde_json::from_str(&h).ok());
            let updated_at = r.updated_at.clone();
            crate::domain::catalog::ItemDto {
                id: r.id,
                sku: r.sku,
                name: r.name,
                category_id: r.category_id,
                unit_id: r.unit_id,
                description: r.description,
                is_active: r.is_active,
                hashtags,
                updated_at,
                created_at: Some(r.updated_at),
                created_by_user_id: None,
                updated_by_user_id: None,
                created_by_user_name: None,
                updated_by_user_name: None,
            }
        }))
    }

    async fn search_items(&self, query: &str) -> CoreResult<Vec<crate::domain::catalog::ItemDto>> {
        let pattern = format!("%{}%", query);
        #[derive(sqlx::FromRow)]
        struct Row {
            id: i32,
            sku: Option<String>,
            name: String,
            category_id: i32,
            unit_id: i32,
            description: Option<String>,
            is_active: bool,
            hashtags: Option<String>,
            updated_at: String,
        }
        let rows: Vec<Row> = sqlx::query_as(
            "SELECT * FROM items WHERE name LIKE ? OR sku LIKE ? ORDER BY name LIMIT 50",
        )
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(rows
            .into_iter()
            .map(|r| {
                let hashtags = r.hashtags.and_then(|h| serde_json::from_str(&h).ok());
                let updated_at = r.updated_at.clone();
                crate::domain::catalog::ItemDto {
                    id: r.id,
                    sku: r.sku,
                    name: r.name,
                    category_id: r.category_id,
                    unit_id: r.unit_id,
                    description: r.description,
                    is_active: r.is_active,
                    hashtags,
                    updated_at,
                    created_at: Some(r.updated_at),
                    created_by_user_id: None,
                    updated_by_user_id: None,
                    created_by_user_name: None,
                    updated_by_user_name: None,
                }
            })
            .collect())
    }

    async fn all_categories(&self) -> CoreResult<Vec<crate::domain::catalog::CategoryDto>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: i32,
            name: String,
            parent_id: Option<i32>,
            is_active: bool,
            updated_at: String,
        }
        let rows: Vec<Row> = sqlx::query_as(
            "SELECT id, name, parent_id, is_active, updated_at FROM categories ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(rows
            .into_iter()
            .map(|r| crate::domain::catalog::CategoryDto {
                id: r.id,
                name: r.name,
                parent_id: r.parent_id,
                is_active: r.is_active,
                updated_at: r.updated_at,
                created_by_user_id: None,
                updated_by_user_id: None,
                created_by_user_name: None,
                updated_by_user_name: None,
            })
            .collect())
    }

    async fn all_units(&self) -> CoreResult<Vec<crate::domain::catalog::UnitDto>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: i32,
            name: String,
            symbol: String,
            is_active: bool,
            updated_at: String,
        }
        let rows: Vec<Row> = sqlx::query_as(
            "SELECT id, name, symbol, is_active, updated_at FROM units ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(rows
            .into_iter()
            .map(|r| crate::domain::catalog::UnitDto {
                id: r.id,
                name: r.name,
                symbol: r.symbol,
                is_active: r.is_active,
                updated_at: r.updated_at,
                created_by_user_id: None,
                updated_by_user_id: None,
                created_by_user_name: None,
                updated_by_user_name: None,
            })
            .collect())
    }

    async fn clear_items(&self) -> CoreResult<()> {
        sqlx::query("DELETE FROM items")
            .execute(&self.pool)
            .await
            .map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn clear_categories(&self) -> CoreResult<()> {
        sqlx::query("DELETE FROM categories")
            .execute(&self.pool)
            .await
            .map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn clear_units(&self) -> CoreResult<()> {
        sqlx::query("DELETE FROM units")
            .execute(&self.pool)
            .await
            .map_err(|e| map_err!(e))?;
        Ok(())
    }
}

// ── BalanceRepo ─────────────────────────────────────────────────

pub trait BalanceRepo {
    async fn upsert_balance(&self, site_id: i32, inv_subject_id: i32, qty: &str) -> CoreResult<()>;
    async fn get_by_item(
        &self,
        item_id: i32,
    ) -> CoreResult<Vec<crate::domain::balance::BalanceRow>>;
    async fn get_by_site(
        &self,
        site_id: i32,
    ) -> CoreResult<Vec<crate::domain::balance::BalanceRow>>;
    async fn get_all(&self) -> CoreResult<Vec<crate::domain::balance::BalanceRow>>;
    async fn clear(&self) -> CoreResult<()>;
}

#[derive(Debug, Clone)]
pub struct SqliteBalanceRepo {
    pool: SqlitePool,
}

impl SqliteBalanceRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl BalanceRepo for SqliteBalanceRepo {
    async fn upsert_balance(&self, site_id: i32, inv_subject_id: i32, qty: &str) -> CoreResult<()> {
        sqlx::query(
            "INSERT INTO balances (site_id, inventory_subject_id, qty, updated_at)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(site_id, inventory_subject_id) DO UPDATE SET
               qty = excluded.qty, updated_at = excluded.updated_at",
        )
        .bind(site_id)
        .bind(inv_subject_id)
        .bind(qty)
        .bind(now_str())
        .execute(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn get_by_item(
        &self,
        item_id: i32,
    ) -> CoreResult<Vec<crate::domain::balance::BalanceRow>> {
        use sqlx::Row;
        let rows = sqlx::query(
            "SELECT b.site_id, b.qty, b.updated_at FROM balances b
             JOIN inventory_subjects s ON b.inventory_subject_id = s.id
             WHERE s.item_id = ? AND s.subject_type = 'catalog_item'",
        )
        .bind(item_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;

        Ok(rows
            .into_iter()
            .map(|r| crate::domain::balance::BalanceRow {
                site_id: r.get(0),
                site_code: String::new(),
                inventory_subject_id: 0,
                subject_type: "catalog_item".to_string(),
                item_id,
                temporary_item_id: None,
                item_name: String::new(),
                item_sku: None,
                unit_symbol: String::new(),
                qty: serde_json::Value::String(r.get::<String, _>(1)),
                updated_at: r.get(2),
                is_subject: None,
            })
            .collect())
    }

    async fn get_by_site(
        &self,
        site_id: i32,
    ) -> CoreResult<Vec<crate::domain::balance::BalanceRow>> {
        use sqlx::Row;
        let rows = sqlx::query(
            "SELECT b.site_id, b.inventory_subject_id, s.subject_type, s.item_id, s.temporary_item_id, b.qty, b.updated_at
             FROM balances b
             LEFT JOIN inventory_subjects s ON b.inventory_subject_id = s.id
             WHERE b.site_id = ?",
        )
        .bind(site_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;

        Ok(rows
            .into_iter()
            .map(|r| crate::domain::balance::BalanceRow {
                site_id: r.get(0),
                site_code: String::new(),
                inventory_subject_id: r.get(1),
                subject_type: r.get::<Option<String>, _>(2).unwrap_or_default(),
                item_id: r.get::<Option<i32>, _>(3).unwrap_or_default(),
                temporary_item_id: r.get(4),
                item_name: String::new(),
                item_sku: None,
                unit_symbol: String::new(),
                qty: serde_json::Value::String(r.get::<String, _>(5)),
                updated_at: r.get(6),
                is_subject: None,
            })
            .collect())
    }

    async fn get_all(&self) -> CoreResult<Vec<crate::domain::balance::BalanceRow>> {
        use sqlx::Row;
        let rows = sqlx::query("SELECT * FROM balances ORDER BY site_id")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| map_err!(e))?;

        Ok(rows
            .into_iter()
            .map(|r| crate::domain::balance::BalanceRow {
                site_id: r.get(0),
                site_code: String::new(),
                inventory_subject_id: r.get(1),
                subject_type: String::new(),
                item_id: 0,
                temporary_item_id: None,
                item_name: String::new(),
                item_sku: None,
                unit_symbol: String::new(),
                qty: serde_json::Value::String(r.get::<String, _>(2)),
                updated_at: r.get(3),
                is_subject: None,
            })
            .collect())
    }

    async fn clear(&self) -> CoreResult<()> {
        sqlx::query("DELETE FROM balances")
            .execute(&self.pool)
            .await
            .map_err(|e| map_err!(e))?;
        Ok(())
    }
}

// ── OutboxRepo ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OutboxEvent {
    pub event_uuid: String,
    pub event_type: String,
    pub command_type: Option<String>,
    pub site_id: i32,
    pub device_id: Option<i32>,
    pub payload: String,
    pub idempotency_key: Option<String>,
    pub payload_hash: Option<String>,
    pub status: String,
    pub retry_count: i32,
    pub max_retries: i32,
    pub last_error: Option<String>,
    pub next_retry_at: Option<String>,
    pub server_result: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub trait OutboxRepo {
    async fn enqueue(
        &self,
        event_type: &str,
        command_type: &str,
        site_id: i32,
        payload: &str,
        idempotency_key: Option<&str>,
        payload_hash: Option<&str>,
    ) -> CoreResult<String>;
    async fn dequeue(&self, batch_size: i32) -> CoreResult<Vec<OutboxEvent>>;
    async fn get(&self, event_uuid: &str) -> CoreResult<Option<OutboxEvent>>;
    async fn list(
        &self,
        site_id: Option<i32>,
        status: Option<&str>,
    ) -> CoreResult<Vec<OutboxEvent>>;
    async fn mark_sending(&self, event_uuid: &str) -> CoreResult<()>;
    async fn mark_success(&self, event_uuid: &str) -> CoreResult<()>;
    async fn mark_failed(&self, event_uuid: &str, error: &str) -> CoreResult<()>;
    async fn mark_conflict(&self, event_uuid: &str, error: &str) -> CoreResult<()>;
    async fn mark_dead_letter(&self, event_uuid: &str, error: &str) -> CoreResult<()>;
    async fn cancel_event(&self, event_uuid: &str) -> CoreResult<()>;
    async fn retry_event(&self, event_uuid: &str) -> CoreResult<()>;
    async fn count_pending(&self) -> CoreResult<i64>;
    async fn count_by_status(&self, status: &str) -> CoreResult<i64>;
    async fn update_server_result(&self, event_uuid: &str, result: &str) -> CoreResult<()>;
}

#[derive(Debug, Clone)]
pub struct SqliteOutboxRepo {
    pool: SqlitePool,
}

impl SqliteOutboxRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl OutboxRepo for SqliteOutboxRepo {
    async fn enqueue(
        &self,
        event_type: &str,
        command_type: &str,
        site_id: i32,
        payload: &str,
        idempotency_key: Option<&str>,
        payload_hash: Option<&str>,
    ) -> CoreResult<String> {
        let uuid = uuid_str();
        sqlx::query(
            "INSERT INTO outbox_events (event_uuid, event_type, command_type, site_id, payload,
             idempotency_key, payload_hash, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&uuid)
        .bind(event_type)
        .bind(command_type)
        .bind(site_id)
        .bind(payload)
        .bind(idempotency_key)
        .bind(payload_hash)
        .bind(now_str())
        .bind(now_str())
        .execute(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(uuid)
    }

    async fn dequeue(&self, batch_size: i32) -> CoreResult<Vec<OutboxEvent>> {
        sqlx::query_as::<_, OutboxEvent>(
            "SELECT * FROM outbox_events
             WHERE status = 'pending'
               AND (next_retry_at IS NULL OR next_retry_at <= datetime('now'))
             ORDER BY created_at ASC LIMIT ?",
        )
        .bind(batch_size)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_err!(e))
    }

    async fn get(&self, event_uuid: &str) -> CoreResult<Option<OutboxEvent>> {
        sqlx::query_as::<_, OutboxEvent>("SELECT * FROM outbox_events WHERE event_uuid = ?")
            .bind(event_uuid)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| map_err!(e))
    }

    async fn list(
        &self,
        site_id: Option<i32>,
        status: Option<&str>,
    ) -> CoreResult<Vec<OutboxEvent>> {
        if let Some(sid) = site_id {
            if let Some(st) = status {
                sqlx::query_as::<_, OutboxEvent>(
                    "SELECT * FROM outbox_events WHERE site_id = ? AND status = ? ORDER BY created_at DESC",
                )
                .bind(sid).bind(st)
                .fetch_all(&self.pool).await.map_err(|e| map_err!(e))
            } else {
                sqlx::query_as::<_, OutboxEvent>(
                    "SELECT * FROM outbox_events WHERE site_id = ? ORDER BY created_at DESC",
                )
                .bind(sid)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| map_err!(e))
            }
        } else if let Some(st) = status {
            sqlx::query_as::<_, OutboxEvent>(
                "SELECT * FROM outbox_events WHERE status = ? ORDER BY created_at DESC",
            )
            .bind(st)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| map_err!(e))
        } else {
            sqlx::query_as::<_, OutboxEvent>("SELECT * FROM outbox_events ORDER BY created_at DESC")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| map_err!(e))
        }
    }

    async fn mark_sending(&self, event_uuid: &str) -> CoreResult<()> {
        sqlx::query(
            "UPDATE outbox_events SET status = 'sending', updated_at = ? WHERE event_uuid = ?",
        )
        .bind(now_str())
        .bind(event_uuid)
        .execute(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn mark_success(&self, event_uuid: &str) -> CoreResult<()> {
        sqlx::query(
            "UPDATE outbox_events SET status = 'accepted', updated_at = ? WHERE event_uuid = ?",
        )
        .bind(now_str())
        .bind(event_uuid)
        .execute(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn mark_failed(&self, event_uuid: &str, error: &str) -> CoreResult<()> {
        sqlx::query(
            "UPDATE outbox_events SET retry_count = retry_count + 1, last_error = ?,
             status = CASE WHEN retry_count + 1 >= max_retries THEN 'failed' ELSE 'pending' END,
             next_retry_at = datetime('now', printf('+%d seconds', MIN(300, (retry_count + 1) * 10))),
             updated_at = ? WHERE event_uuid = ?",
        )
        .bind(error).bind(now_str()).bind(event_uuid)
        .execute(&self.pool).await.map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn mark_conflict(&self, event_uuid: &str, error: &str) -> CoreResult<()> {
        sqlx::query(
            "UPDATE outbox_events SET status = 'conflict', last_error = ?, updated_at = ? WHERE event_uuid = ?",
        )
        .bind(error).bind(now_str()).bind(event_uuid)
        .execute(&self.pool).await.map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn mark_dead_letter(&self, event_uuid: &str, error: &str) -> CoreResult<()> {
        sqlx::query(
            "UPDATE outbox_events SET status = 'dead_letter', last_error = ?, updated_at = ? WHERE event_uuid = ?",
        )
        .bind(error).bind(now_str()).bind(event_uuid)
        .execute(&self.pool).await.map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn cancel_event(&self, event_uuid: &str) -> CoreResult<()> {
        sqlx::query(
            "UPDATE outbox_events SET status = 'cancelled', updated_at = ? WHERE event_uuid = ?",
        )
        .bind(now_str())
        .bind(event_uuid)
        .execute(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn retry_event(&self, event_uuid: &str) -> CoreResult<()> {
        sqlx::query(
            "UPDATE outbox_events SET status = 'pending', retry_count = 0, last_error = NULL,
             next_retry_at = NULL, updated_at = ? WHERE event_uuid = ?",
        )
        .bind(now_str())
        .bind(event_uuid)
        .execute(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn count_pending(&self) -> CoreResult<i64> {
        sqlx::query_scalar("SELECT COUNT(*) FROM outbox_events WHERE status = 'pending'")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| map_err!(e))
    }

    async fn count_by_status(&self, status: &str) -> CoreResult<i64> {
        sqlx::query_scalar("SELECT COUNT(*) FROM outbox_events WHERE status = ?")
            .bind(status)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| map_err!(e))
    }

    async fn update_server_result(&self, event_uuid: &str, result: &str) -> CoreResult<()> {
        sqlx::query(
            "UPDATE outbox_events SET server_result = ?, updated_at = ? WHERE event_uuid = ?",
        )
        .bind(result)
        .bind(now_str())
        .bind(event_uuid)
        .execute(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(())
    }
}

// ── SyncCursorRepo ──────────────────────────────────────────────

pub trait SyncCursorRepo {
    async fn get_cursor(&self, cursor_type: &str) -> CoreResult<Option<String>>;
    async fn set_cursor(&self, cursor_type: &str, value: &str) -> CoreResult<()>;
}

#[derive(Debug, Clone)]
pub struct SqliteSyncCursorRepo {
    pub(crate) pool: SqlitePool,
}

impl SqliteSyncCursorRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl SyncCursorRepo for SqliteSyncCursorRepo {
    async fn get_cursor(&self, cursor_type: &str) -> CoreResult<Option<String>> {
        sqlx::query_scalar::<_, String>(
            "SELECT cursor_value FROM sync_cursors WHERE cursor_type = ?",
        )
        .bind(cursor_type)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| map_err!(e))
    }

    async fn set_cursor(&self, cursor_type: &str, value: &str) -> CoreResult<()> {
        sqlx::query(
            "INSERT INTO sync_cursors (cursor_type, cursor_value, updated_at) VALUES (?, ?, ?)
             ON CONFLICT(cursor_type) DO UPDATE SET cursor_value = excluded.cursor_value, updated_at = excluded.updated_at",
        )
        .bind(cursor_type).bind(value).bind(now_str())
        .execute(&self.pool).await.map_err(|e| map_err!(e))?;
        Ok(())
    }
}

// ── DraftRepo ───────────────────────────────────────────────────

pub trait DraftRepo {
    async fn save(&self, draft: &crate::domain::operation::OperationDraft) -> CoreResult<()>;
    async fn get(
        &self,
        draft_id: &str,
    ) -> CoreResult<Option<crate::domain::operation::OperationDraft>>;
    async fn list(&self) -> CoreResult<Vec<crate::domain::operation::OperationDraft>>;
    async fn delete(&self, draft_id: &str) -> CoreResult<()>;
}

#[derive(Debug, Clone)]
pub struct SqliteDraftRepo {
    pool: SqlitePool,
}

impl SqliteDraftRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl DraftRepo for SqliteDraftRepo {
    async fn save(&self, draft: &crate::domain::operation::OperationDraft) -> CoreResult<()> {
        let mut tx = self.pool.begin().await.map_err(|e| map_err!(e))?;

        sqlx::query(
            "INSERT INTO operation_drafts (draft_id, operation_type, site_id, effective_at,
             source_site_id, destination_site_id, recipient_id, issued_to_name, comment, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(draft_id) DO UPDATE SET
               operation_type = excluded.operation_type, site_id = excluded.site_id,
               effective_at = excluded.effective_at, source_site_id = excluded.source_site_id,
               destination_site_id = excluded.destination_site_id, recipient_id = excluded.recipient_id,
               issued_to_name = excluded.issued_to_name, comment = excluded.comment, updated_at = excluded.updated_at",
        )
            .bind(draft.draft_id.to_string())                                    // 1: draft_id
            .bind(serde_json::to_string(&draft.operation_type).unwrap_or_default()) // 2: operation_type
            .bind(draft.site_id)                                                  // 3: site_id
            .bind(&draft.effective_at)                                            // 4: effective_at
            .bind(draft.source_site_id)                                           // 5: source_site_id
            .bind(draft.destination_site_id)                                      // 6: destination_site_id
            .bind(draft.recipient_id)                                             // 7: recipient_id
            .bind(&draft.issued_to_name)                                          // 8: issued_to_name
            .bind(&draft.comment)                                                 // 9: comment
            .bind(&draft.created_at)                                              // 10: created_at
            .bind(&draft.updated_at)                                              // 11: updated_at
            .execute(&mut *tx).await.map_err(|e| map_err!(e))?;

        sqlx::query("DELETE FROM operation_draft_lines WHERE draft_id = ?")
            .bind(draft.draft_id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| map_err!(e))?;

        for (i, line) in draft.lines.iter().enumerate() {
            let temp = &line.temporary_item;
            sqlx::query(
                "INSERT INTO operation_draft_lines
                 (line_id, draft_id, item_id, temp_name, temp_sku, temp_category_id, temp_unit_id,
                  temp_description, qty, batch, comment, sort_order)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(line.line_id.to_string())
            .bind(draft.draft_id.to_string())
            .bind(line.item_id)
            .bind(temp.as_ref().map(|t| &t.name))
            .bind(temp.as_ref().and_then(|t| t.sku.as_ref()))
            .bind(temp.as_ref().and_then(|t| t.category_id))
            .bind(temp.as_ref().map(|t| t.unit_id))
            .bind(temp.as_ref().and_then(|t| t.description.as_ref()))
            .bind(line.qty.to_string())
            .bind(&line.batch)
            .bind(&line.comment)
            .bind(i as i32)
            .execute(&mut *tx)
            .await
            .map_err(|e| map_err!(e))?;
        }

        tx.commit().await.map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn get(
        &self,
        draft_id: &str,
    ) -> CoreResult<Option<crate::domain::operation::OperationDraft>> {
        #[derive(sqlx::FromRow)]
        struct DraftRow {
            draft_id: String,
            operation_type: String,
            site_id: Option<i32>,
            effective_at: Option<String>,
            source_site_id: Option<i32>,
            destination_site_id: Option<i32>,
            recipient_id: Option<i32>,
            issued_to_name: Option<String>,
            comment: Option<String>,
            created_at: String,
            updated_at: String,
        }
        let draft: Option<DraftRow> =
            sqlx::query_as("SELECT * FROM operation_drafts WHERE draft_id = ?")
                .bind(draft_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| map_err!(e))?;

        let draft = match draft {
            Some(d) => d,
            None => return Ok(None),
        };

        #[derive(sqlx::FromRow)]
        struct LineRow {
            line_id: String,
            item_id: Option<i32>,
            temp_name: Option<String>,
            temp_sku: Option<String>,
            temp_category_id: Option<i32>,
            temp_unit_id: Option<i32>,
            temp_description: Option<String>,
            qty: String,
            batch: Option<String>,
            comment: Option<String>,
        }
        let lines: Vec<LineRow> = sqlx::query_as(
            "SELECT * FROM operation_draft_lines WHERE draft_id = ? ORDER BY sort_order",
        )
        .bind(draft_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;

        Ok(Some(crate::domain::operation::OperationDraft {
            draft_id: uuid::Uuid::parse_str(&draft.draft_id).unwrap_or_default(),
            operation_type: serde_json::from_str(&format!("\"{}\"", draft.operation_type))
                .unwrap_or_default(),
            site_id: draft.site_id,
            lines: lines
                .into_iter()
                .map(|l| crate::domain::operation::OperationDraftLine {
                    line_id: uuid::Uuid::parse_str(&l.line_id).unwrap_or_default(),
                    item_id: l.item_id,
                    temporary_item: l.temp_name.map(|n| {
                        crate::domain::operation::TemporaryItemInlineCreate {
                            name: n,
                            sku: l.temp_sku,
                            category_id: l.temp_category_id,
                            unit_id: l.temp_unit_id.unwrap_or(0),
                            description: l.temp_description,
                        }
                    }),
                    qty: serde_json::Value::String(l.qty),
                    batch: l.batch,
                    comment: l.comment,
                })
                .collect(),
            effective_at: draft.effective_at,
            source_site_id: draft.source_site_id,
            destination_site_id: draft.destination_site_id,
            recipient_id: draft.recipient_id,
            issued_to_name: draft.issued_to_name,
            comment: draft.comment,
            created_at: draft.created_at,
            updated_at: draft.updated_at,
        }))
    }

    async fn list(&self) -> CoreResult<Vec<crate::domain::operation::OperationDraft>> {
        // Simple: list drafts without lines
        #[derive(sqlx::FromRow)]
        struct DraftRow {
            draft_id: String,
            operation_type: String,
            site_id: Option<i32>,
            effective_at: Option<String>,
            source_site_id: Option<i32>,
            destination_site_id: Option<i32>,
            recipient_id: Option<i32>,
            issued_to_name: Option<String>,
            comment: Option<String>,
            created_at: String,
            updated_at: String,
        }
        let drafts: Vec<DraftRow> =
            sqlx::query_as("SELECT * FROM operation_drafts ORDER BY updated_at DESC")
                .fetch_all(&self.pool)
                .await
                .map_err(|e| map_err!(e))?;

        let mut result = Vec::new();
        for d in drafts {
            let item = crate::domain::operation::OperationDraft {
                draft_id: uuid::Uuid::parse_str(&d.draft_id).unwrap_or_default(),
                operation_type: serde_json::from_str(&format!("\"{}\"", d.operation_type))
                    .unwrap_or_default(),
                site_id: d.site_id,
                lines: Vec::new(),
                effective_at: d.effective_at,
                source_site_id: d.source_site_id,
                destination_site_id: d.destination_site_id,
                recipient_id: d.recipient_id,
                issued_to_name: d.issued_to_name,
                comment: d.comment,
                created_at: d.created_at,
                updated_at: d.updated_at,
            };
            result.push(item);
        }
        Ok(result)
    }

    async fn delete(&self, draft_id: &str) -> CoreResult<()> {
        let mut tx = self.pool.begin().await.map_err(|e| map_err!(e))?;
        sqlx::query("DELETE FROM operation_draft_lines WHERE draft_id = ?")
            .bind(draft_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| map_err!(e))?;
        sqlx::query("DELETE FROM operation_drafts WHERE draft_id = ?")
            .bind(draft_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| map_err!(e))?;
        tx.commit().await.map_err(|e| map_err!(e))?;
        Ok(())
    }
}

// ── ErrorLogRepo ────────────────────────────────────────────────

pub trait ErrorLogRepo {
    async fn log(
        &self,
        level: &str,
        category: &str,
        message: &str,
        details: Option<&str>,
    ) -> CoreResult<i64>;
    async fn recent(
        &self,
        limit: i32,
    ) -> CoreResult<Vec<(i64, String, String, String, Option<String>, String)>>;
}

#[derive(Debug, Clone)]
pub struct SqliteErrorLogRepo {
    pool: SqlitePool,
}

impl SqliteErrorLogRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl ErrorLogRepo for SqliteErrorLogRepo {
    async fn log(
        &self,
        level: &str,
        category: &str,
        message: &str,
        details: Option<&str>,
    ) -> CoreResult<i64> {
        let id = sqlx::query(
            "INSERT INTO error_log (level, category, message, details) VALUES (?, ?, ?, ?)",
        )
        .bind(level)
        .bind(category)
        .bind(message)
        .bind(details)
        .execute(&self.pool)
        .await
        .map_err(|e| map_err!(e))?
        .last_insert_rowid();
        Ok(id)
    }

    async fn recent(
        &self,
        limit: i32,
    ) -> CoreResult<Vec<(i64, String, String, String, Option<String>, String)>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            error_id: i64,
            level: String,
            category: String,
            message: String,
            details: Option<String>,
            created_at: String,
        }
        let rows: Vec<Row> =
            sqlx::query_as("SELECT * FROM error_log ORDER BY error_id DESC LIMIT ?")
                .bind(limit)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| map_err!(e))?;
        Ok(rows
            .into_iter()
            .map(|r| {
                (
                    r.error_id,
                    r.level,
                    r.category,
                    r.message,
                    r.details,
                    r.created_at,
                )
            })
            .collect())
    }
}

// ── AssetsRepo ──────────────────────────────────────────────────

pub trait AssetsRepo {
    async fn upsert_pending_acceptance(
        &self,
        row: &crate::domain::assets::PendingAcceptanceRow,
        site_id: i32,
        inv_subject_id: i32,
    ) -> CoreResult<()>;
    async fn get_pending_acceptance(
        &self,
    ) -> CoreResult<Vec<crate::domain::assets::PendingAcceptanceRow>>;
    async fn clear_pending_acceptance(&self) -> CoreResult<()>;
    async fn upsert_lost_asset(
        &self,
        row: &crate::domain::assets::LostAssetRow,
        site_id: i32,
        inv_subject_id: i32,
    ) -> CoreResult<()>;
    async fn get_lost_assets(&self) -> CoreResult<Vec<crate::domain::assets::LostAssetRow>>;
    async fn clear_lost_assets(&self) -> CoreResult<()>;
    async fn upsert_issued_asset(
        &self,
        row: &crate::domain::assets::IssuedAssetRow,
        site_id: i32,
        inv_subject_id: i32,
    ) -> CoreResult<()>;
    async fn get_issued_assets(&self) -> CoreResult<Vec<crate::domain::assets::IssuedAssetRow>>;
    async fn clear_issued_assets(&self) -> CoreResult<()>;
}

#[derive(Debug, Clone)]
pub struct SqliteAssetsRepo {
    pool: SqlitePool,
}

impl SqliteAssetsRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl AssetsRepo for SqliteAssetsRepo {
    async fn upsert_pending_acceptance(
        &self,
        row: &crate::domain::assets::PendingAcceptanceRow,
        site_id: i32,
        inv_subject_id: i32,
    ) -> CoreResult<()> {
        let qty = row.qty.to_string();
        let accepted = row.accepted_qty.as_ref().map(|v| v.to_string());
        let lost = row.lost_qty.as_ref().map(|v| v.to_string());
        sqlx::query(
            "INSERT INTO pending_acceptance_balances
             (operation_id, operation_line_id, site_id, inventory_subject_id, qty, accepted_qty, lost_qty, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(operation_line_id) DO UPDATE SET
               operation_id = excluded.operation_id, site_id = excluded.site_id,
               inventory_subject_id = excluded.inventory_subject_id, qty = excluded.qty,
               accepted_qty = excluded.accepted_qty, lost_qty = excluded.lost_qty,
               updated_at = excluded.updated_at",
        )
        .bind(&row.operation_id)
        .bind(&row.operation_line_id)
        .bind(site_id)
        .bind(inv_subject_id)
        .bind(&qty)
        .bind(&accepted)
        .bind(&lost)
        .bind(now_str())
        .execute(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn get_pending_acceptance(
        &self,
    ) -> CoreResult<Vec<crate::domain::assets::PendingAcceptanceRow>> {
        use sqlx::Row;
        let rows = sqlx::query("SELECT * FROM pending_acceptance_balances ORDER BY operation_id")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| map_err!(e))?;
        Ok(rows
            .into_iter()
            .map(|r| crate::domain::assets::PendingAcceptanceRow {
                operation_id: r.get(0),
                operation_line_id: r.get(1),
                item_id: 0,
                item_name: String::new(),
                item_sku: None,
                unit_symbol: String::new(),
                qty: serde_json::Value::String(r.get::<String, _>(4)),
                accepted_qty: r.get::<Option<String>, _>(5).map(serde_json::Value::String),
                lost_qty: r.get::<Option<String>, _>(6).map(serde_json::Value::String),
                destination_site_id: None,
                source_site_id: None,
                inventory_subject_id: None,
                subject_type: None,
            })
            .collect())
    }

    async fn clear_pending_acceptance(&self) -> CoreResult<()> {
        sqlx::query("DELETE FROM pending_acceptance_balances")
            .execute(&self.pool)
            .await
            .map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn upsert_lost_asset(
        &self,
        row: &crate::domain::assets::LostAssetRow,
        site_id: i32,
        inv_subject_id: i32,
    ) -> CoreResult<()> {
        let lost = row.lost_qty.to_string();
        sqlx::query(
            "INSERT INTO lost_asset_balances
             (operation_line_id, operation_id, site_id, inventory_subject_id, lost_qty, is_resolved, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(operation_line_id) DO UPDATE SET
               operation_id = excluded.operation_id, site_id = excluded.site_id,
               inventory_subject_id = excluded.inventory_subject_id, lost_qty = excluded.lost_qty,
               is_resolved = excluded.is_resolved, updated_at = excluded.updated_at",
        )
        .bind(&row.operation_line_id)
        .bind(&row.operation_id)
        .bind(site_id)
        .bind(inv_subject_id)
        .bind(&lost)
        .bind(row.is_resolved)
        .bind(now_str())
        .execute(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn get_lost_assets(&self) -> CoreResult<Vec<crate::domain::assets::LostAssetRow>> {
        use sqlx::Row;
        let rows = sqlx::query("SELECT * FROM lost_asset_balances ORDER BY operation_id")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| map_err!(e))?;
        Ok(rows
            .into_iter()
            .map(|r| crate::domain::assets::LostAssetRow {
                operation_id: r.get(1),
                operation_line_id: r.get(0),
                item_id: 0,
                item_name: String::new(),
                item_sku: None,
                unit_symbol: String::new(),
                qty: serde_json::Value::String(r.get::<String, _>(4)),
                lost_qty: serde_json::Value::String(r.get::<String, _>(4)),
                is_resolved: r.get(5),
            })
            .collect())
    }

    async fn clear_lost_assets(&self) -> CoreResult<()> {
        sqlx::query("DELETE FROM lost_asset_balances")
            .execute(&self.pool)
            .await
            .map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn upsert_issued_asset(
        &self,
        row: &crate::domain::assets::IssuedAssetRow,
        site_id: i32,
        inv_subject_id: i32,
    ) -> CoreResult<()> {
        let qty = row.qty.to_string();
        sqlx::query(
            "INSERT INTO issued_asset_balances
             (operation_line_id, operation_id, site_id, inventory_subject_id, qty, issued_to_name, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(operation_line_id) DO UPDATE SET
               operation_id = excluded.operation_id, site_id = excluded.site_id,
               inventory_subject_id = excluded.inventory_subject_id, qty = excluded.qty,
               issued_to_name = excluded.issued_to_name, updated_at = excluded.updated_at",
        )
        .bind(&row.operation_line_id)
        .bind(&row.operation_id)
        .bind(site_id)
        .bind(inv_subject_id)
        .bind(&qty)
        .bind(&row.issued_to_name)
        .bind(now_str())
        .execute(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn get_issued_assets(&self) -> CoreResult<Vec<crate::domain::assets::IssuedAssetRow>> {
        use sqlx::Row;
        let rows = sqlx::query("SELECT * FROM issued_asset_balances ORDER BY operation_id")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| map_err!(e))?;
        Ok(rows
            .into_iter()
            .map(|r| crate::domain::assets::IssuedAssetRow {
                operation_id: r.get(1),
                operation_line_id: r.get(0),
                item_id: 0,
                item_name: String::new(),
                item_sku: None,
                unit_symbol: String::new(),
                qty: serde_json::Value::String(r.get::<String, _>(4)),
                issued_to_name: r.get(5),
            })
            .collect())
    }

    async fn clear_issued_assets(&self) -> CoreResult<()> {
        sqlx::query("DELETE FROM issued_asset_balances")
            .execute(&self.pool)
            .await
            .map_err(|e| map_err!(e))?;
        Ok(())
    }
}

// ── RecipientRepo ───────────────────────────────────────────────

pub trait RecipientRepo {
    async fn upsert(&self, r: &crate::domain::recipient::RecipientDto) -> CoreResult<()>;
    async fn get(&self, id: i32) -> CoreResult<Option<crate::domain::recipient::RecipientDto>>;
    async fn list(&self) -> CoreResult<Vec<crate::domain::recipient::RecipientDto>>;
    async fn search(&self, query: &str) -> CoreResult<Vec<crate::domain::recipient::RecipientDto>>;
    async fn delete_by_id(&self, id: i32) -> CoreResult<()>;
    async fn clear(&self) -> CoreResult<()>;
}

#[derive(Debug, Clone)]
pub struct SqliteRecipientRepo {
    pool: SqlitePool,
}

impl SqliteRecipientRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl RecipientRepo for SqliteRecipientRepo {
    async fn upsert(&self, r: &crate::domain::recipient::RecipientDto) -> CoreResult<()> {
        sqlx::query(
            "INSERT INTO recipients (id, name, recipient_type, contact_info, is_active, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               name = excluded.name, recipient_type = excluded.recipient_type,
               contact_info = excluded.contact_info, is_active = excluded.is_active,
               updated_at = excluded.updated_at",
        )
        .bind(r.id)
        .bind(&r.name)
        .bind(serde_json::to_string(&r.recipient_type).unwrap_or_default())
        .bind(&r.contact_info)
        .bind(r.is_active)
        .bind(&r.created_at)
        .bind(&r.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn get(&self, id: i32) -> CoreResult<Option<crate::domain::recipient::RecipientDto>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: i32,
            name: String,
            recipient_type: String,
            contact_info: Option<String>,
            is_active: bool,
            created_at: String,
            updated_at: String,
        }
        let row: Option<Row> = sqlx::query_as("SELECT * FROM recipients WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| map_err!(e))?;
        Ok(row.map(|r| crate::domain::recipient::RecipientDto {
            id: r.id,
            name: r.name,
            recipient_type: serde_json::from_str(&r.recipient_type).unwrap_or_default(),
            contact_info: r.contact_info,
            is_active: r.is_active,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }))
    }

    async fn list(&self) -> CoreResult<Vec<crate::domain::recipient::RecipientDto>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: i32,
            name: String,
            recipient_type: String,
            contact_info: Option<String>,
            is_active: bool,
            created_at: String,
            updated_at: String,
        }
        let rows: Vec<Row> = sqlx::query_as("SELECT * FROM recipients ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| map_err!(e))?;
        Ok(rows
            .into_iter()
            .map(|r| crate::domain::recipient::RecipientDto {
                id: r.id,
                name: r.name,
                recipient_type: serde_json::from_str(&r.recipient_type).unwrap_or_default(),
                contact_info: r.contact_info,
                is_active: r.is_active,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect())
    }

    async fn search(&self, query: &str) -> CoreResult<Vec<crate::domain::recipient::RecipientDto>> {
        let pattern = format!("%{}%", query);
        #[derive(sqlx::FromRow)]
        struct Row {
            id: i32,
            name: String,
            recipient_type: String,
            contact_info: Option<String>,
            is_active: bool,
            created_at: String,
            updated_at: String,
        }
        let rows: Vec<Row> =
            sqlx::query_as("SELECT * FROM recipients WHERE name LIKE ? ORDER BY name LIMIT 50")
                .bind(&pattern)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| map_err!(e))?;
        Ok(rows
            .into_iter()
            .map(|r| crate::domain::recipient::RecipientDto {
                id: r.id,
                name: r.name,
                recipient_type: serde_json::from_str(&r.recipient_type).unwrap_or_default(),
                contact_info: r.contact_info,
                is_active: r.is_active,
                created_at: r.created_at,
                updated_at: r.updated_at,
            })
            .collect())
    }

    async fn delete_by_id(&self, id: i32) -> CoreResult<()> {
        sqlx::query("DELETE FROM recipients WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn clear(&self) -> CoreResult<()> {
        sqlx::query("DELETE FROM recipients")
            .execute(&self.pool)
            .await
            .map_err(|e| map_err!(e))?;
        Ok(())
    }
}

// ── TemporaryItemRepo ───────────────────────────────────────────

pub trait TemporaryItemRepo {
    async fn upsert(
        &self,
        item: &crate::domain::temporary_items::TemporaryItemDto,
    ) -> CoreResult<()>;
    async fn get(
        &self,
        id: i32,
    ) -> CoreResult<Option<crate::domain::temporary_items::TemporaryItemDto>>;
    async fn list_active(
        &self,
    ) -> CoreResult<Vec<crate::domain::temporary_items::TemporaryItemDto>>;
    async fn clear(&self) -> CoreResult<()>;
}

#[derive(Debug, Clone)]
pub struct SqliteTemporaryItemRepo {
    pool: SqlitePool,
}

impl SqliteTemporaryItemRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl TemporaryItemRepo for SqliteTemporaryItemRepo {
    async fn upsert(
        &self,
        item: &crate::domain::temporary_items::TemporaryItemDto,
    ) -> CoreResult<()> {
        sqlx::query(
            "INSERT INTO temporary_items
             (id, name, sku, category_id, unit_id, description, status, resolved_item_id, created_by_user_id, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               name = excluded.name, sku = excluded.sku, category_id = excluded.category_id,
               unit_id = excluded.unit_id, description = excluded.description, status = excluded.status,
               resolved_item_id = excluded.resolved_item_id, created_by_user_id = excluded.created_by_user_id,
               updated_at = excluded.updated_at",
        )
        .bind(item.id)
        .bind(&item.name)
        .bind(&item.sku)
        .bind(item.category_id)
        .bind(item.unit_id)
        .bind(&item.description)
        .bind(serde_json::to_string(&item.status).unwrap_or_default())
        .bind(item.resolved_item_id)
        .bind(item.created_by_user_id.to_string())
        .bind(&item.created_at)
        .bind(&item.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn get(
        &self,
        id: i32,
    ) -> CoreResult<Option<crate::domain::temporary_items::TemporaryItemDto>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: i32,
            name: String,
            sku: Option<String>,
            category_id: Option<i32>,
            unit_id: i32,
            description: Option<String>,
            status: String,
            resolved_item_id: Option<i32>,
            created_by_user_id: String,
            created_at: String,
            updated_at: String,
        }
        let row: Option<Row> = sqlx::query_as("SELECT * FROM temporary_items WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| map_err!(e))?;
        Ok(row.map(|r| {
            let status: crate::domain::temporary_items::TemporaryItemStatus =
                serde_json::from_str(&r.status).unwrap_or_default();
            crate::domain::temporary_items::TemporaryItemDto {
                id: r.id,
                name: r.name,
                sku: r.sku,
                category_id: r.category_id,
                unit_id: r.unit_id,
                description: r.description,
                status,
                resolved_item_id: r.resolved_item_id,
                resolved_item_name: None,
                created_by_user_id: uuid::Uuid::parse_str(&r.created_by_user_id)
                    .unwrap_or_default(),
                created_at: r.created_at,
                updated_at: r.updated_at,
            }
        }))
    }

    async fn list_active(
        &self,
    ) -> CoreResult<Vec<crate::domain::temporary_items::TemporaryItemDto>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: i32,
            name: String,
            sku: Option<String>,
            category_id: Option<i32>,
            unit_id: i32,
            description: Option<String>,
            status: String,
            resolved_item_id: Option<i32>,
            created_by_user_id: String,
            created_at: String,
            updated_at: String,
        }
        let rows: Vec<Row> = sqlx::query_as(
            "SELECT * FROM temporary_items WHERE status = '\"active\"' ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(rows
            .into_iter()
            .map(|r| {
                let status: crate::domain::temporary_items::TemporaryItemStatus =
                    serde_json::from_str(&r.status).unwrap_or_default();
                crate::domain::temporary_items::TemporaryItemDto {
                    id: r.id,
                    name: r.name,
                    sku: r.sku,
                    category_id: r.category_id,
                    unit_id: r.unit_id,
                    description: r.description,
                    status,
                    resolved_item_id: r.resolved_item_id,
                    resolved_item_name: None,
                    created_by_user_id: uuid::Uuid::parse_str(&r.created_by_user_id)
                        .unwrap_or_default(),
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                }
            })
            .collect())
    }

    async fn clear(&self) -> CoreResult<()> {
        sqlx::query("DELETE FROM temporary_items")
            .execute(&self.pool)
            .await
            .map_err(|e| map_err!(e))?;
        Ok(())
    }
}

// ── DocumentRepo ────────────────────────────────────────────────

pub trait DocumentRepo {
    async fn upsert(&self, doc: &crate::domain::documents::DocumentDto) -> CoreResult<()>;
    async fn get(&self, id: &str) -> CoreResult<Option<crate::domain::documents::DocumentDto>>;
    async fn list_by_site(
        &self,
        site_id: i32,
    ) -> CoreResult<Vec<crate::domain::documents::DocumentDto>>;
    async fn clear(&self) -> CoreResult<()>;
}

#[derive(Debug, Clone)]
pub struct SqliteDocumentRepo {
    pool: SqlitePool,
}

impl SqliteDocumentRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl DocumentRepo for SqliteDocumentRepo {
    async fn upsert(&self, doc: &crate::domain::documents::DocumentDto) -> CoreResult<()> {
        let payload = doc.payload.as_ref().map(|p| p.to_string());
        let created_by = doc.created_by_user_id.as_ref().map(|u| u.to_string());
        sqlx::query(
            "INSERT INTO documents_cache
             (id, document_type, status, document_number, revision, site_id,
              template_name, template_version, payload, created_by_user_id,
              created_at, finalized_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
               document_type = excluded.document_type, status = excluded.status,
               document_number = excluded.document_number, revision = excluded.revision,
               site_id = excluded.site_id, template_name = excluded.template_name,
               template_version = excluded.template_version, payload = excluded.payload,
               created_by_user_id = excluded.created_by_user_id,
               finalized_at = excluded.finalized_at, updated_at = excluded.updated_at",
        )
        .bind(doc.id.to_string())
        .bind(serde_json::to_string(&doc.document_type).unwrap_or_default())
        .bind(serde_json::to_string(&doc.status).unwrap_or_default())
        .bind(&doc.document_number)
        .bind(doc.revision)
        .bind(doc.site_id)
        .bind(&doc.template_name)
        .bind(&doc.template_version)
        .bind(&payload)
        .bind(&created_by)
        .bind(&doc.created_at)
        .bind(&doc.finalized_at)
        .bind(now_str())
        .execute(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn get(&self, id: &str) -> CoreResult<Option<crate::domain::documents::DocumentDto>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: String,
            document_type: String,
            status: String,
            document_number: Option<String>,
            revision: i32,
            site_id: i32,
            template_name: Option<String>,
            template_version: Option<String>,
            payload: Option<String>,
            created_by_user_id: Option<String>,
            created_at: String,
            finalized_at: Option<String>,
        }
        let row: Option<Row> = sqlx::query_as("SELECT * FROM documents_cache WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| map_err!(e))?;
        Ok(row.map(|r| {
            let payload = r.payload.and_then(|p| serde_json::from_str(&p).ok());
            let created_by = r
                .created_by_user_id
                .and_then(|u| uuid::Uuid::parse_str(&u).ok());
            crate::domain::documents::DocumentDto {
                id: uuid::Uuid::parse_str(&r.id).unwrap_or_default(),
                document_type: serde_json::from_str(&r.document_type).unwrap_or_default(),
                status: serde_json::from_str(&r.status).unwrap_or_default(),
                document_number: r.document_number,
                revision: r.revision,
                site_id: r.site_id,
                template_name: r.template_name,
                template_version: r.template_version,
                payload,
                created_by_user_id: created_by,
                created_at: r.created_at,
                finalized_at: r.finalized_at,
            }
        }))
    }

    async fn list_by_site(
        &self,
        site_id: i32,
    ) -> CoreResult<Vec<crate::domain::documents::DocumentDto>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            id: String,
            document_type: String,
            status: String,
            document_number: Option<String>,
            revision: i32,
            site_id: i32,
            template_name: Option<String>,
            template_version: Option<String>,
            payload: Option<String>,
            created_by_user_id: Option<String>,
            created_at: String,
            finalized_at: Option<String>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            "SELECT * FROM documents_cache WHERE site_id = ? ORDER BY created_at DESC",
        )
        .bind(site_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(rows
            .into_iter()
            .map(|r| {
                let payload = r.payload.and_then(|p| serde_json::from_str(&p).ok());
                let created_by = r
                    .created_by_user_id
                    .and_then(|u| uuid::Uuid::parse_str(&u).ok());
                crate::domain::documents::DocumentDto {
                    id: uuid::Uuid::parse_str(&r.id).unwrap_or_default(),
                    document_type: serde_json::from_str(&r.document_type).unwrap_or_default(),
                    status: serde_json::from_str(&r.status).unwrap_or_default(),
                    document_number: r.document_number,
                    revision: r.revision,
                    site_id: r.site_id,
                    template_name: r.template_name,
                    template_version: r.template_version,
                    payload,
                    created_by_user_id: created_by,
                    created_at: r.created_at,
                    finalized_at: r.finalized_at,
                }
            })
            .collect())
    }

    async fn clear(&self) -> CoreResult<()> {
        sqlx::query("DELETE FROM documents_cache")
            .execute(&self.pool)
            .await
            .map_err(|e| map_err!(e))?;
        Ok(())
    }
}

// ── ReportsRepo ─────────────────────────────────────────────────

pub trait ReportsRepo {
    async fn cache_item_movement(
        &self,
        params_hash: &str,
        rows: &[crate::domain::reports::ItemMovementRow],
    ) -> CoreResult<()>;
    async fn get_cached_item_movement(
        &self,
        params_hash: &str,
    ) -> CoreResult<Vec<crate::domain::reports::ItemMovementRow>>;
    async fn cache_stock_summary(
        &self,
        params_hash: &str,
        rows: &[crate::domain::reports::StockSummaryRow],
    ) -> CoreResult<()>;
    async fn get_cached_stock_summary(
        &self,
        params_hash: &str,
    ) -> CoreResult<Vec<crate::domain::reports::StockSummaryRow>>;
}

#[derive(Debug, Clone)]
pub struct SqliteReportsRepo {
    pool: SqlitePool,
}

impl SqliteReportsRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl ReportsRepo for SqliteReportsRepo {
    async fn cache_item_movement(
        &self,
        params_hash: &str,
        rows: &[crate::domain::reports::ItemMovementRow],
    ) -> CoreResult<()> {
        let mut tx = self.pool.begin().await.map_err(|e| map_err!(e))?;
        sqlx::query("DELETE FROM report_item_movement_cache WHERE params_hash = ?")
            .bind(params_hash)
            .execute(&mut *tx)
            .await
            .map_err(|e| map_err!(e))?;
        let now = now_str();
        for r in rows {
            let qty = r.quantity.to_string();
            sqlx::query(
                "INSERT INTO report_item_movement_cache
                 (params_hash, item_id, item_name, item_sku, unit_symbol, operation_type,
                  operation_id, quantity, effective_at, site_id, site_code, refreshed_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(params_hash)
            .bind(r.item_id)
            .bind(&r.item_name)
            .bind(&r.item_sku)
            .bind(&r.unit_symbol)
            .bind(&r.operation_type)
            .bind(&r.operation_id)
            .bind(&qty)
            .bind(&r.effective_at)
            .bind(r.site_id)
            .bind(&r.site_code)
            .bind(&now)
            .execute(&mut *tx)
            .await
            .map_err(|e| map_err!(e))?;
        }
        tx.commit().await.map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn get_cached_item_movement(
        &self,
        params_hash: &str,
    ) -> CoreResult<Vec<crate::domain::reports::ItemMovementRow>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            item_id: i32,
            item_name: String,
            item_sku: Option<String>,
            unit_symbol: String,
            operation_type: String,
            operation_id: String,
            quantity: String,
            effective_at: String,
            site_id: i32,
            site_code: String,
        }
        let rows: Vec<Row> = sqlx::query_as(
            "SELECT item_id, item_name, item_sku, unit_symbol, operation_type, operation_id,
                    quantity, effective_at, site_id, site_code
             FROM report_item_movement_cache WHERE params_hash = ? ORDER BY effective_at",
        )
        .bind(params_hash)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(rows
            .into_iter()
            .map(|r| crate::domain::reports::ItemMovementRow {
                item_id: r.item_id,
                item_name: r.item_name,
                item_sku: r.item_sku,
                unit_symbol: r.unit_symbol,
                operation_type: r.operation_type,
                operation_id: r.operation_id,
                quantity: serde_json::Value::String(r.quantity),
                effective_at: r.effective_at,
                site_id: r.site_id,
                site_code: r.site_code,
            })
            .collect())
    }

    async fn cache_stock_summary(
        &self,
        params_hash: &str,
        rows: &[crate::domain::reports::StockSummaryRow],
    ) -> CoreResult<()> {
        let mut tx = self.pool.begin().await.map_err(|e| map_err!(e))?;
        sqlx::query("DELETE FROM report_stock_summary_cache WHERE params_hash = ?")
            .bind(params_hash)
            .execute(&mut *tx)
            .await
            .map_err(|e| map_err!(e))?;
        let now = now_str();
        for r in rows {
            let qty = r.quantity.to_string();
            sqlx::query(
                "INSERT INTO report_stock_summary_cache
                 (params_hash, item_id, item_name, item_sku, unit_symbol, site_id,
                  site_code, quantity, last_operation_at, refreshed_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(params_hash)
            .bind(r.item_id)
            .bind(&r.item_name)
            .bind(&r.item_sku)
            .bind(&r.unit_symbol)
            .bind(r.site_id)
            .bind(&r.site_code)
            .bind(&qty)
            .bind(&r.last_operation_at)
            .bind(&now)
            .execute(&mut *tx)
            .await
            .map_err(|e| map_err!(e))?;
        }
        tx.commit().await.map_err(|e| map_err!(e))?;
        Ok(())
    }

    async fn get_cached_stock_summary(
        &self,
        params_hash: &str,
    ) -> CoreResult<Vec<crate::domain::reports::StockSummaryRow>> {
        #[derive(sqlx::FromRow)]
        struct Row {
            item_id: i32,
            item_name: String,
            item_sku: Option<String>,
            unit_symbol: String,
            site_id: i32,
            site_code: String,
            quantity: String,
            last_operation_at: Option<String>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            "SELECT item_id, item_name, item_sku, unit_symbol, site_id, site_code,
                    quantity, last_operation_at
             FROM report_stock_summary_cache WHERE params_hash = ? ORDER BY item_name",
        )
        .bind(params_hash)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(rows
            .into_iter()
            .map(|r| crate::domain::reports::StockSummaryRow {
                item_id: r.item_id,
                item_name: r.item_name,
                item_sku: r.item_sku,
                unit_symbol: r.unit_symbol,
                site_id: r.site_id,
                site_code: r.site_code,
                quantity: serde_json::Value::String(r.quantity),
                last_operation_at: r.last_operation_at,
            })
            .collect())
    }
}

// ── SyncRunRepo ──────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SqliteSyncRunRepo {
    pool: SqlitePool,
}

impl SqliteSyncRunRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn insert_sync_run(&self, summary: &crate::sync::SyncRunSummary) -> CoreResult<()> {
        let status = if summary.errors_count > 0 {
            "completed_with_errors"
        } else {
            "completed"
        };
        let families_json = serde_json::to_string(&summary.families).map_err(|e| map_err!(e))?;
        let mode = "pull"; // default when backfilling from PullSyncService
        sqlx::query(
            "INSERT OR REPLACE INTO sync_runs (run_id, started_at, finished_at, status, push_count, pull_count, error_count, error, families_json, mode)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&summary.run_id)
        .bind(&summary.started_at)
        .bind(&summary.completed_at)
        .bind(status)
        .bind(0i64) // push_count not tracked by pull-only
        .bind(summary.total_items as i64)
        .bind(summary.errors_count as i64)
        .bind(summary.errors().first().map(|s| s.to_string()))
        .bind(&families_json)
        .bind(mode)
        .execute(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;
        Ok(())
    }

    pub async fn list_sync_runs(&self, limit: i32) -> CoreResult<Vec<crate::sync::SyncRunSummary>> {
        #[derive(sqlx::FromRow)]
        #[allow(dead_code)]
        struct Row {
            run_id: String,
            started_at: String,
            finished_at: Option<String>,
            status: String,
            pull_count: i64,
            error_count: i64,
            error: Option<String>,
            families_json: Option<String>,
            mode: Option<String>,
        }
        let rows: Vec<Row> = sqlx::query_as(
            "SELECT run_id, started_at, finished_at, status, pull_count, error_count, error, families_json, mode
             FROM sync_runs ORDER BY started_at DESC LIMIT ?",
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| map_err!(e))?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let families_json = r.families_json.unwrap_or_default();
                let families: Vec<crate::sync::FamilyResult> =
                    serde_json::from_str(&families_json).unwrap_or_default();
                let is_complete = r.status != "running";
                crate::sync::SyncRunSummary {
                    run_id: r.run_id,
                    started_at: r.started_at,
                    completed_at: r.finished_at,
                    families,
                    total_items: r.pull_count as usize,
                    errors_count: r.error_count as usize,
                    is_complete,
                }
            })
            .collect())
    }
}

// ── RepoBag: all repos in one struct ────────────────────────────

#[derive(Clone)]
pub struct RepoBag {
    pub auth_context: SqliteAuthContextRepo,
    pub catalog: SqliteCatalogRepo,
    pub balances: SqliteBalanceRepo,
    pub outbox: SqliteOutboxRepo,
    pub sync_cursors: SqliteSyncCursorRepo,
    pub drafts: SqliteDraftRepo,
    pub errors: SqliteErrorLogRepo,
    pub assets: SqliteAssetsRepo,
    pub recipients: SqliteRecipientRepo,
    pub temporary_items: SqliteTemporaryItemRepo,
    pub documents: SqliteDocumentRepo,
    pub reports: SqliteReportsRepo,
}

impl RepoBag {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            auth_context: SqliteAuthContextRepo::new(pool.clone()),
            catalog: SqliteCatalogRepo::new(pool.clone()),
            balances: SqliteBalanceRepo::new(pool.clone()),
            outbox: SqliteOutboxRepo::new(pool.clone()),
            sync_cursors: SqliteSyncCursorRepo::new(pool.clone()),
            drafts: SqliteDraftRepo::new(pool.clone()),
            errors: SqliteErrorLogRepo::new(pool.clone()),
            assets: SqliteAssetsRepo::new(pool.clone()),
            recipients: SqliteRecipientRepo::new(pool.clone()),
            temporary_items: SqliteTemporaryItemRepo::new(pool.clone()),
            documents: SqliteDocumentRepo::new(pool.clone()),
            reports: SqliteReportsRepo::new(pool),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::operation::{OperationDraft, OperationDraftLine, OperationType};

    async fn test_pool() -> SqlitePool {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create test pool");
        crate::storage::migrations::run_migrations(&pool)
            .await
            .expect("Migrations failed");
        pool
    }

    // ── CatalogRepo tests ─────────────────────────────────────────

    #[tokio::test]
    async fn test_catalog_insert_and_get_unit() {
        let pool = test_pool().await;
        let repo = SqliteCatalogRepo::new(pool);
        let unit = crate::domain::catalog::UnitDto {
            id: 1,
            name: "штука".into(),
            symbol: "шт".into(),
            is_active: true,
            updated_at: "2024-01-01T00:00:00Z".into(),
            created_by_user_id: None,
            updated_by_user_id: None,
            created_by_user_name: None,
            updated_by_user_name: None,
        };
        repo.upsert_unit(&unit).await.unwrap();
        let all = repo.all_units().await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "штука");
        assert_eq!(all[0].symbol, "шт");
    }

    #[tokio::test]
    async fn test_catalog_insert_and_get_category() {
        let pool = test_pool().await;
        let repo = SqliteCatalogRepo::new(pool);
        let cat = crate::domain::catalog::CategoryDto {
            id: 1,
            name: "Электроника".into(),
            parent_id: None,
            is_active: true,
            updated_at: "2024-01-01T00:00:00Z".into(),
            created_by_user_id: None,
            updated_by_user_id: None,
            created_by_user_name: None,
            updated_by_user_name: None,
        };
        repo.upsert_category(&cat).await.unwrap();
        let all = repo.all_categories().await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "Электроника");
    }

    #[tokio::test]
    async fn test_catalog_insert_and_get_item() {
        let pool = test_pool().await;
        let repo = SqliteCatalogRepo::new(pool);
        let unit = crate::domain::catalog::UnitDto {
            id: 1,
            name: "штука".into(),
            symbol: "шт".into(),
            is_active: true,
            updated_at: "2024-01-01T00:00:00Z".into(),
            created_by_user_id: None,
            updated_by_user_id: None,
            created_by_user_name: None,
            updated_by_user_name: None,
        };
        let cat = crate::domain::catalog::CategoryDto {
            id: 1,
            name: "Инструменты".into(),
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
        let item = crate::domain::catalog::ItemDto {
            id: 1,
            sku: Some("SKU-001".into()),
            name: "Молоток".into(),
            category_id: 1,
            unit_id: 1,
            description: Some("Ударный инструмент".into()),
            is_active: true,
            hashtags: Some(vec!["tool".into()]),
            updated_at: "2024-01-01T00:00:00Z".into(),
            created_at: Some("2024-01-01T00:00:00Z".into()),
            created_by_user_id: None,
            updated_by_user_id: None,
            created_by_user_name: None,
            updated_by_user_name: None,
        };
        repo.upsert_item(&item).await.unwrap();
        let got = repo.get_item(1).await.unwrap().expect("Item not found");
        assert_eq!(got.name, "Молоток");
        assert_eq!(got.sku, Some("SKU-001".into()));
        assert_eq!(got.category_id, 1);
        assert_eq!(got.unit_id, 1);
    }

    #[tokio::test]
    async fn test_catalog_search_items() {
        let pool = test_pool().await;
        let repo = SqliteCatalogRepo::new(pool);
        let unit = crate::domain::catalog::UnitDto {
            id: 1,
            name: "штука".into(),
            symbol: "шт".into(),
            is_active: true,
            updated_at: "2024-01-01T00:00:00Z".into(),
            created_by_user_id: None,
            updated_by_user_id: None,
            created_by_user_name: None,
            updated_by_user_name: None,
        };
        let cat = crate::domain::catalog::CategoryDto {
            id: 1,
            name: "Test".into(),
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
        for i in 1..=5 {
            let item = crate::domain::catalog::ItemDto {
                id: i,
                sku: Some(format!("SKU-{i:03}")),
                name: format!("Searchable Item {i}"),
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
        let results = repo.search_items("Searchable").await.unwrap();
        assert_eq!(results.len(), 5);
    }

    #[tokio::test]
    async fn test_catalog_search_items_no_match() {
        let pool = test_pool().await;
        let repo = SqliteCatalogRepo::new(pool);
        let results = repo.search_items("NonExistent").await.unwrap();
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_catalog_all_categories() {
        let pool = test_pool().await;
        let repo = SqliteCatalogRepo::new(pool);
        for i in 1..=3 {
            let cat = crate::domain::catalog::CategoryDto {
                id: i,
                name: format!("Cat {i}"),
                parent_id: if i > 1 { Some(1) } else { None },
                is_active: true,
                updated_at: "2024-01-01T00:00:00Z".into(),
                created_by_user_id: None,
                updated_by_user_id: None,
                created_by_user_name: None,
                updated_by_user_name: None,
            };
            repo.upsert_category(&cat).await.unwrap();
        }
        let all = repo.all_categories().await.unwrap();
        assert_eq!(all.len(), 3);
    }

    #[tokio::test]
    async fn test_catalog_all_units() {
        let pool = test_pool().await;
        let repo = SqliteCatalogRepo::new(pool);
        for i in 1..=3 {
            let unit = crate::domain::catalog::UnitDto {
                id: i,
                name: format!("Unit {i}"),
                symbol: format!("U{i}"),
                is_active: true,
                updated_at: "2024-01-01T00:00:00Z".into(),
                created_by_user_id: None,
                updated_by_user_id: None,
                created_by_user_name: None,
                updated_by_user_name: None,
            };
            repo.upsert_unit(&unit).await.unwrap();
        }
        let all = repo.all_units().await.unwrap();
        assert_eq!(all.len(), 3);
    }

    // ── BalanceRepo tests ─────────────────────────────────────────

    #[tokio::test]
    async fn test_balance_insert_and_get_by_site() {
        let pool = test_pool().await;
        let repo = SqliteBalanceRepo::new(pool.clone());
        sqlx::query("INSERT INTO inventory_subjects (id, item_id, subject_type) VALUES (?, ?, ?)")
            .bind(1i32)
            .bind(1i32)
            .bind("catalog_item")
            .execute(&pool)
            .await
            .unwrap();
        repo.upsert_balance(1, 1, "10.5").await.unwrap();
        let rows = repo.get_by_site(1).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].qty, serde_json::json!("10.5"));
    }

    #[tokio::test]
    async fn test_balance_get_by_item() {
        let pool = test_pool().await;
        let repo = SqliteBalanceRepo::new(pool.clone());
        sqlx::query("INSERT INTO inventory_subjects (id, item_id, subject_type) VALUES (?, ?, ?)")
            .bind(1i32)
            .bind(42i32)
            .bind("catalog_item")
            .execute(&pool)
            .await
            .unwrap();
        repo.upsert_balance(1, 1, "7").await.unwrap();
        let rows = repo.get_by_item(42).await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].site_id, 1);
    }

    #[tokio::test]
    async fn test_balance_empty_for_site() {
        let pool = test_pool().await;
        let repo = SqliteBalanceRepo::new(pool);
        let rows = repo.get_by_site(999).await.unwrap();
        assert!(rows.is_empty());
    }

    // ── AssetsRepo tests ──────────────────────────────────────────

    #[tokio::test]
    async fn test_pending_acceptance_crud() {
        let pool = test_pool().await;
        let repo = SqliteAssetsRepo::new(pool.clone());
        sqlx::query("INSERT INTO inventory_subjects (id, item_id, subject_type) VALUES (?, ?, ?)")
            .bind(1i32)
            .bind(1i32)
            .bind("catalog_item")
            .execute(&pool)
            .await
            .unwrap();
        let row = crate::domain::assets::PendingAcceptanceRow {
            operation_id: "op-1".into(),
            operation_line_id: "line-1".into(),
            item_id: 1,
            item_name: "Test Asset".into(),
            item_sku: None,
            unit_symbol: "шт".into(),
            qty: serde_json::json!("5"),
            accepted_qty: Some(serde_json::json!("3")),
            lost_qty: None,
            destination_site_id: None,
            source_site_id: None,
            inventory_subject_id: Some(1),
            subject_type: Some("catalog_item".into()),
        };
        repo.upsert_pending_acceptance(&row, 1, 1).await.unwrap();
        let list = repo.get_pending_acceptance().await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].operation_id, "op-1");
    }

    #[tokio::test]
    async fn test_lost_assets_crud() {
        let pool = test_pool().await;
        let repo = SqliteAssetsRepo::new(pool.clone());
        sqlx::query("INSERT INTO inventory_subjects (id, item_id, subject_type) VALUES (?, ?, ?)")
            .bind(1i32)
            .bind(1i32)
            .bind("catalog_item")
            .execute(&pool)
            .await
            .unwrap();
        let row = crate::domain::assets::LostAssetRow {
            operation_id: "op-2".into(),
            operation_line_id: "line-2".into(),
            item_id: 1,
            item_name: "Lost Item".into(),
            item_sku: None,
            unit_symbol: "шт".into(),
            qty: serde_json::json!("2"),
            lost_qty: serde_json::json!("2"),
            is_resolved: false,
        };
        repo.upsert_lost_asset(&row, 1, 1).await.unwrap();
        let list = repo.get_lost_assets().await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].operation_id, "op-2");
    }

    #[tokio::test]
    async fn test_issued_assets_crud() {
        let pool = test_pool().await;
        let repo = SqliteAssetsRepo::new(pool.clone());
        sqlx::query("INSERT INTO inventory_subjects (id, item_id, subject_type) VALUES (?, ?, ?)")
            .bind(1i32)
            .bind(1i32)
            .bind("catalog_item")
            .execute(&pool)
            .await
            .unwrap();
        let row = crate::domain::assets::IssuedAssetRow {
            operation_id: "op-3".into(),
            operation_line_id: "line-3".into(),
            item_id: 1,
            item_name: "Issued Item".into(),
            item_sku: None,
            unit_symbol: "шт".into(),
            qty: serde_json::json!("1"),
            issued_to_name: "Иван".into(),
        };
        repo.upsert_issued_asset(&row, 1, 1).await.unwrap();
        let list = repo.get_issued_assets().await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].issued_to_name, "Иван");
    }

    // ── RecipientRepo tests ───────────────────────────────────────

    #[tokio::test]
    async fn test_recipient_insert_and_get() {
        let pool = test_pool().await;
        let repo = SqliteRecipientRepo::new(pool);
        let r = crate::domain::recipient::RecipientDto {
            id: 1,
            name: "ООО Поставщик".into(),
            recipient_type: crate::domain::recipient::RecipientType::Contractor,
            contact_info: Some("contact@supplier.ru".into()),
            is_active: true,
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        };
        repo.upsert(&r).await.unwrap();
        let got = repo.get(1).await.unwrap().expect("Recipient not found");
        assert_eq!(got.name, "ООО Поставщик");
        assert_eq!(
            got.recipient_type,
            crate::domain::recipient::RecipientType::Contractor
        );
    }

    #[tokio::test]
    async fn test_recipient_search() {
        let pool = test_pool().await;
        let repo = SqliteRecipientRepo::new(pool);
        for i in 1..=3 {
            let r = crate::domain::recipient::RecipientDto {
                id: i,
                name: format!("SearchRecipient {i}"),
                recipient_type: crate::domain::recipient::RecipientType::Person,
                contact_info: None,
                is_active: true,
                created_at: "2024-01-01T00:00:00Z".into(),
                updated_at: "2024-01-01T00:00:00Z".into(),
            };
            repo.upsert(&r).await.unwrap();
        }
        let results = repo.search("SearchRecipient").await.unwrap();
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_recipient_not_found() {
        let pool = test_pool().await;
        let repo = SqliteRecipientRepo::new(pool);
        let got = repo.get(999).await.unwrap();
        assert!(got.is_none());
    }

    #[tokio::test]
    async fn test_recipient_delete() {
        let pool = test_pool().await;
        let repo = SqliteRecipientRepo::new(pool);
        let r = crate::domain::recipient::RecipientDto {
            id: 1,
            name: "ToDelete".into(),
            recipient_type: crate::domain::recipient::RecipientType::Person,
            contact_info: None,
            is_active: true,
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        };
        repo.upsert(&r).await.unwrap();
        repo.delete_by_id(1).await.unwrap();
        let got = repo.get(1).await.unwrap();
        assert!(got.is_none());
    }

    // ── DraftRepo tests ───────────────────────────────────────────

    fn sample_draft() -> OperationDraft {
        OperationDraft {
            draft_id: uuid::Uuid::new_v4(),
            operation_type: OperationType::Receive,
            site_id: Some(1),
            lines: vec![OperationDraftLine {
                line_id: uuid::Uuid::new_v4(),
                item_id: Some(1),
                temporary_item: None,
                qty: serde_json::json!("10"),
                batch: None,
                comment: None,
            }],
            effective_at: None,
            source_site_id: None,
            destination_site_id: None,
            recipient_id: None,
            issued_to_name: None,
            comment: None,
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
        }
    }

    #[tokio::test]
    async fn test_draft_create_and_get() {
        let pool = test_pool().await;
        let repo = SqliteDraftRepo::new(pool);
        let draft = sample_draft();
        repo.save(&draft).await.unwrap();
        let got = repo
            .get(&draft.draft_id.to_string())
            .await
            .unwrap()
            .expect("Draft not found");
        assert_eq!(got.operation_type, OperationType::Receive);
        assert_eq!(got.site_id, Some(1));
        assert_eq!(got.lines.len(), 1);
    }

    #[tokio::test]
    async fn test_draft_list() {
        let pool = test_pool().await;
        let repo = SqliteDraftRepo::new(pool);
        for _ in 0..3 {
            repo.save(&sample_draft()).await.unwrap();
        }
        let list = repo.list().await.unwrap();
        assert_eq!(list.len(), 3);
    }

    #[tokio::test]
    async fn test_draft_delete() {
        let pool = test_pool().await;
        let repo = SqliteDraftRepo::new(pool);
        let draft = sample_draft();
        repo.save(&draft).await.unwrap();
        repo.delete(&draft.draft_id.to_string()).await.unwrap();
        let got = repo.get(&draft.draft_id.to_string()).await.unwrap();
        assert!(got.is_none());
    }

    #[tokio::test]
    async fn test_draft_clone() {
        let pool = test_pool().await;
        let repo = SqliteDraftRepo::new(pool);
        let draft = sample_draft();
        repo.save(&draft).await.unwrap();
        let original_id = draft.draft_id;

        // Clone manually: new UUID, same lines
        let cloned = OperationDraft {
            draft_id: uuid::Uuid::new_v4(),
            created_at: "2024-02-01T00:00:00Z".into(),
            updated_at: "2024-02-01T00:00:00Z".into(),
            lines: draft
                .lines
                .into_iter()
                .map(|l| OperationDraftLine {
                    line_id: uuid::Uuid::new_v4(),
                    ..l
                })
                .collect(),
            ..draft
        };
        repo.save(&cloned).await.unwrap();

        let orig_got = repo.get(&original_id.to_string()).await.unwrap();
        assert!(orig_got.is_some());
        let clone_got = repo
            .get(&cloned.draft_id.to_string())
            .await
            .unwrap()
            .expect("Clone not found");
        assert_eq!(clone_got.lines.len(), 1);
        assert_ne!(clone_got.draft_id, original_id);
    }

    // ── OutboxRepo tests ──────────────────────────────────────────

    #[tokio::test]
    async fn test_outbox_enqueue_and_list() {
        let pool = test_pool().await;
        let repo = SqliteOutboxRepo::new(pool);
        repo.enqueue("test_type", "test_cmd", 1, r#"{"key":"val"}"#, None, None)
            .await
            .unwrap();
        repo.enqueue(
            "test_type2",
            "test_cmd2",
            1,
            r#"{"key2":"val2"}"#,
            None,
            None,
        )
        .await
        .unwrap();
        let list = repo.list(Some(1), None).await.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_outbox_mark_sending() {
        let pool = test_pool().await;
        let repo = SqliteOutboxRepo::new(pool);
        let uuid = repo
            .enqueue("op", "cmd", 1, "{}", None, None)
            .await
            .unwrap();
        repo.mark_sending(&uuid).await.unwrap();
        let ev = repo.get(&uuid).await.unwrap().expect("Event not found");
        assert_eq!(ev.status, "sending");
    }

    #[tokio::test]
    async fn test_outbox_mark_success() {
        let pool = test_pool().await;
        let repo = SqliteOutboxRepo::new(pool);
        let uuid = repo
            .enqueue("op", "cmd", 1, "{}", None, None)
            .await
            .unwrap();
        repo.mark_sending(&uuid).await.unwrap();
        repo.mark_success(&uuid).await.unwrap();
        let ev = repo.get(&uuid).await.unwrap().expect("Event not found");
        assert_eq!(ev.status, "accepted");
    }

    #[tokio::test]
    async fn test_outbox_mark_failed() {
        let pool = test_pool().await;
        let repo = SqliteOutboxRepo::new(pool);
        let uuid = repo
            .enqueue("op", "cmd", 1, "{}", None, None)
            .await
            .unwrap();
        repo.mark_failed(&uuid, "timeout").await.unwrap();
        let ev = repo.get(&uuid).await.unwrap().expect("Event not found");
        assert_eq!(ev.retry_count, 1);
        assert_eq!(ev.last_error.as_deref(), Some("timeout"));
    }

    #[tokio::test]
    async fn test_outbox_mark_conflict() {
        let pool = test_pool().await;
        let repo = SqliteOutboxRepo::new(pool);
        let uuid = repo
            .enqueue("op", "cmd", 1, "{}", None, None)
            .await
            .unwrap();
        repo.mark_conflict(&uuid, "conflict").await.unwrap();
        let ev = repo.get(&uuid).await.unwrap().expect("Event not found");
        assert_eq!(ev.status, "conflict");
        assert_eq!(ev.last_error.as_deref(), Some("conflict"));
    }

    #[tokio::test]
    async fn test_outbox_cancel() {
        let pool = test_pool().await;
        let repo = SqliteOutboxRepo::new(pool);
        let uuid = repo
            .enqueue("op", "cmd", 1, "{}", None, None)
            .await
            .unwrap();
        repo.cancel_event(&uuid).await.unwrap();
        let ev = repo.get(&uuid).await.unwrap().expect("Event not found");
        assert_eq!(ev.status, "cancelled");
    }

    #[tokio::test]
    async fn test_outbox_retry() {
        let pool = test_pool().await;
        let repo = SqliteOutboxRepo::new(pool);
        let uuid = repo
            .enqueue("op", "cmd", 1, "{}", None, None)
            .await
            .unwrap();
        repo.mark_failed(&uuid, "err").await.unwrap();
        repo.retry_event(&uuid).await.unwrap();
        let ev = repo.get(&uuid).await.unwrap().expect("Event not found");
        assert_eq!(ev.status, "pending");
        assert_eq!(ev.retry_count, 0);
        assert!(ev.last_error.is_none());
    }

    #[tokio::test]
    async fn test_outbox_count_pending() {
        let pool = test_pool().await;
        let repo = SqliteOutboxRepo::new(pool);
        for _ in 0..5 {
            repo.enqueue("op", "cmd", 1, "{}", None, None)
                .await
                .unwrap();
        }
        let count = repo.count_pending().await.unwrap();
        assert_eq!(count, 5);
        // Change one to sending, verify count drops
        let list = repo.list(None, Some("pending")).await.unwrap();
        repo.mark_sending(&list[0].event_uuid).await.unwrap();
        let count2 = repo.count_pending().await.unwrap();
        assert_eq!(count2, 4);
    }

    // ── SyncRunRepo tests ─────────────────────────────────────────

    #[tokio::test]
    async fn test_sync_run_insert_and_list() {
        let pool = test_pool().await;
        let repo = SqliteSyncRunRepo::new(pool);
        let summary = crate::sync::SyncRunSummary {
            run_id: "run-1".into(),
            started_at: "2024-01-01T00:00:00Z".into(),
            completed_at: Some("2024-01-01T00:01:00Z".into()),
            families: vec![],
            total_items: 10,
            errors_count: 0,
            is_complete: true,
        };
        repo.insert_sync_run(&summary).await.unwrap();
        let list = repo.list_sync_runs(10).await.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].run_id, "run-1");
        assert_eq!(list[0].total_items, 10);
    }

    #[tokio::test]
    async fn test_sync_run_list_empty() {
        let pool = test_pool().await;
        let repo = SqliteSyncRunRepo::new(pool);
        let list = repo.list_sync_runs(10).await.unwrap();
        assert!(list.is_empty());
    }

    #[tokio::test]
    async fn test_sync_run_multiple_inserts() {
        let pool = test_pool().await;
        let repo = SqliteSyncRunRepo::new(pool);
        let runs = vec![
            ("run-a", "2024-01-01T00:00:00Z"),
            ("run-b", "2024-01-02T00:00:00Z"),
            ("run-c", "2024-01-03T00:00:00Z"),
        ];
        for (rid, ts) in &runs {
            let summary = crate::sync::SyncRunSummary {
                run_id: rid.to_string(),
                started_at: ts.to_string(),
                completed_at: None,
                families: vec![],
                total_items: 0,
                errors_count: 0,
                is_complete: false,
            };
            repo.insert_sync_run(&summary).await.unwrap();
        }
        let list = repo.list_sync_runs(10).await.unwrap();
        assert_eq!(list.len(), 3);
        // Ordered by started_at DESC
        assert_eq!(list[0].run_id, "run-c");
        assert_eq!(list[1].run_id, "run-b");
        assert_eq!(list[2].run_id, "run-a");
    }
}
