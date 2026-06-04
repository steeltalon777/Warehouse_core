use crate::domain::assets::{IssuedAssetRow, LostAssetRow, PendingAcceptanceRow};
use crate::domain::balance::BalanceRow;
use crate::domain::catalog::{CatalogSiteDto, CategoryDto, ItemDto, UnitDto};
use crate::domain::documents::DocumentDto;
use crate::domain::operation::OperationListItem;
use crate::domain::recipient::RecipientDto;
use crate::domain::reports::StockSummaryRow;
use crate::domain::temporary_items::TemporaryItemDto;
use crate::error::CoreResult;
use crate::storage::repos::{ReportsRepo, SqliteReportsRepo};
use sqlx::SqlitePool;

fn now_str() -> String {
    crate::time::Timestamp::now_utc().to_string()
}

/// SnapshotWriter: transactional writes for each pull family.
/// Snapshot families that are fetched as full lists replace local state atomically.
/// Cursor/delta families merge by primary key to avoid wiping unchanged parents.
pub struct SnapshotWriter {
    pool: SqlitePool,
}

impl SnapshotWriter {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    async fn table_has_rows(&self, table: &str) -> CoreResult<bool> {
        let query = format!("SELECT 1 FROM {table} LIMIT 1");
        let row = sqlx::query(&query)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        Ok(row.is_some())
    }

    pub async fn has_items(&self) -> CoreResult<bool> {
        self.table_has_rows("items").await
    }

    pub async fn has_categories(&self) -> CoreResult<bool> {
        self.table_has_rows("categories").await
    }

    pub async fn has_units(&self) -> CoreResult<bool> {
        self.table_has_rows("units").await
    }

    pub async fn write_items(&self, items: &[ItemDto]) -> CoreResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        for item in items {
            let hashtags = item
                .hashtags
                .as_ref()
                .map(|h| serde_json::to_string(h).unwrap_or_default());
            let created_at = item.created_at.as_deref().unwrap_or(&item.updated_at);
            sqlx::query(
                "INSERT INTO items (id, sku, name, category_id, unit_id, description, is_active, hashtags, updated_at, created_at,
                                    created_by_user_id, updated_by_user_id, created_by_user_name, updated_by_user_name)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
                         ?, ?, ?, ?)
                 ON CONFLICT(id) DO UPDATE SET
                   sku = excluded.sku,
                   name = excluded.name,
                   category_id = excluded.category_id,
                   unit_id = excluded.unit_id,
                   description = excluded.description,
                   is_active = excluded.is_active,
                   hashtags = excluded.hashtags,
                   updated_at = excluded.updated_at,
                   updated_by_user_id = excluded.updated_by_user_id,
                   updated_by_user_name = excluded.updated_by_user_name",
            )
            .bind(item.id).bind(&item.sku).bind(&item.name)
            .bind(item.category_id).bind(item.unit_id).bind(&item.description)
            .bind(item.is_active).bind(&hashtags).bind(&item.updated_at)
            .bind(created_at)
            .bind(&item.created_by_user_id).bind(&item.updated_by_user_id)
            .bind(&item.created_by_user_name).bind(&item.updated_by_user_name)
            .execute(&mut *tx).await.map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        }
        tx.commit()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        Ok(())
    }

    pub async fn write_categories(&self, cats: &[CategoryDto]) -> CoreResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        let now = now_str();
        for cat in cats {
            sqlx::query(
                "INSERT INTO categories (id, name, parent_id, is_active, created_at, updated_at,
                                         created_by_user_id, updated_by_user_id, created_by_user_name, updated_by_user_name)
                 VALUES (?, ?, ?, ?, ?, ?,
                         ?, ?, ?, ?)
                 ON CONFLICT(id) DO UPDATE SET
                   name = excluded.name,
                   parent_id = excluded.parent_id,
                   is_active = excluded.is_active,
                   updated_at = excluded.updated_at,
                   updated_by_user_id = excluded.updated_by_user_id,
                   updated_by_user_name = excluded.updated_by_user_name",
            )
            .bind(cat.id)
            .bind(&cat.name)
            .bind(cat.parent_id)
            .bind(cat.is_active)
            .bind(&now)
            .bind(&cat.updated_at)
            .bind(&cat.created_by_user_id).bind(&cat.updated_by_user_id)
            .bind(&cat.created_by_user_name).bind(&cat.updated_by_user_name)
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        }
        tx.commit()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        Ok(())
    }

    pub async fn write_units(&self, units: &[UnitDto]) -> CoreResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        let now = now_str();
        for unit in units {
            sqlx::query(
                "INSERT INTO units (id, name, symbol, is_active, created_at, updated_at,
                                    created_by_user_id, updated_by_user_id, created_by_user_name, updated_by_user_name)
                 VALUES (?, ?, ?, ?, ?, ?,
                         ?, ?, ?, ?)
                 ON CONFLICT(id) DO UPDATE SET
                   name = excluded.name,
                   symbol = excluded.symbol,
                   is_active = excluded.is_active,
                   updated_at = excluded.updated_at,
                   updated_by_user_id = excluded.updated_by_user_id,
                   updated_by_user_name = excluded.updated_by_user_name",
            )
            .bind(unit.id)
            .bind(&unit.name)
            .bind(&unit.symbol)
            .bind(unit.is_active)
            .bind(&now)
            .bind(&unit.updated_at)
            .bind(&unit.created_by_user_id).bind(&unit.updated_by_user_id)
            .bind(&unit.created_by_user_name).bind(&unit.updated_by_user_name)
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        }
        tx.commit()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        Ok(())
    }

    pub async fn write_sites(&self, sites: &[CatalogSiteDto]) -> CoreResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        sqlx::query("DELETE FROM sites")
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        for s in sites {
            sqlx::query("INSERT INTO sites (site_id, code, name, is_active, updated_at) VALUES (?, ?, ?, ?, ?)")
                .bind(s.site_id)
                .bind(&s.code)
                .bind(&s.name)
                .bind(s.is_active)
                .bind(now_str())
                .execute(&mut *tx)
                .await
                .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        }
        tx.commit()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        Ok(())
    }

    pub async fn write_balances(&self, site_id: i32, rows: &[BalanceRow]) -> CoreResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        sqlx::query("DELETE FROM balances WHERE site_id = ?")
            .bind(site_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        let now = now_str();
        for r in rows {
            sqlx::query(
                "INSERT INTO inventory_subjects (id, subject_type, item_id, temporary_item_id, created_at)
                 VALUES (?, ?, ?, ?, ?)
                 ON CONFLICT(id) DO UPDATE SET
                   subject_type = excluded.subject_type,
                   item_id = excluded.item_id,
                   temporary_item_id = excluded.temporary_item_id",
            )
            .bind(r.inventory_subject_id)
            .bind(&r.subject_type)
            .bind(if r.item_id == 0 { None } else { Some(r.item_id) })
            .bind(r.temporary_item_id)
            .bind(&now)
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
            let qty_str = r.qty.to_string();
            sqlx::query(
                "INSERT INTO balances (site_id, inventory_subject_id, qty, updated_at) VALUES (?, ?, ?, ?)",
            )
            .bind(site_id)
            .bind(r.inventory_subject_id)
            .bind(&qty_str)
            .bind(&now)
            .execute(&mut *tx).await.map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        }
        tx.commit()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        Ok(())
    }

    pub async fn write_recipients(&self, recipients: &[RecipientDto]) -> CoreResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        sqlx::query("DELETE FROM recipients")
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        for r in recipients {
            sqlx::query(
                "INSERT INTO recipients (id, name, recipient_type, contact_info, is_active, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(r.id).bind(&r.name)
            .bind(serde_json::to_string(&r.recipient_type).unwrap_or_default())
            .bind(&r.contact_info).bind(r.is_active).bind(&r.created_at).bind(&r.updated_at)
            .execute(&mut *tx).await.map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        }
        tx.commit()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        Ok(())
    }

    pub async fn write_temporary_items(&self, items: &[TemporaryItemDto]) -> CoreResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        sqlx::query("DELETE FROM temporary_items")
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        for item in items {
            sqlx::query(
                "INSERT INTO temporary_items
                 (id, name, sku, category_id, unit_id, description, status, resolved_item_id, created_by_user_id, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(item.id).bind(&item.name).bind(&item.sku)
            .bind(item.category_id).bind(item.unit_id).bind(&item.description)
            .bind(serde_json::to_string(&item.status).unwrap_or_default())
            .bind(item.resolved_item_id)
            .bind(item.created_by_user_id.to_string())
            .bind(&item.created_at).bind(&item.updated_at)
            .execute(&mut *tx).await.map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        }
        tx.commit()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        Ok(())
    }

    pub async fn write_operations(&self, ops: &[OperationListItem]) -> CoreResult<()> {
        let _ = ops;
        Ok(())
    }

    pub async fn write_documents(&self, docs: &[DocumentDto]) -> CoreResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        sqlx::query("DELETE FROM documents_cache")
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        let now = now_str();
        for doc in docs {
            let payload = doc.payload.as_ref().map(|p| p.to_string());
            let created_by = doc.created_by_user_id.as_ref().map(|u| u.to_string());
            sqlx::query(
                "INSERT INTO documents_cache
                 (id, document_type, status, document_number, revision, site_id,
                  template_name, template_version, payload, created_by_user_id,
                  created_at, finalized_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
            .bind(&now)
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        }
        tx.commit()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        Ok(())
    }

    pub async fn write_pending_acceptance(&self, rows: &[PendingAcceptanceRow]) -> CoreResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        sqlx::query("DELETE FROM pending_acceptance_balances")
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        let now = now_str();
        for row in rows {
            let qty = row.qty.to_string();
            let accepted = row.accepted_qty.as_ref().map(|v| v.to_string());
            let lost = row.lost_qty.as_ref().map(|v| v.to_string());
            sqlx::query(
                "INSERT INTO pending_acceptance_balances
                 (operation_id, operation_line_id, site_id, inventory_subject_id, qty, accepted_qty, lost_qty, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&row.operation_id).bind(&row.operation_line_id)
            .bind(0).bind(0)
            .bind(&qty).bind(&accepted).bind(&lost).bind(&now)
            .execute(&mut *tx).await.map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        }
        tx.commit()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        Ok(())
    }

    pub async fn write_lost_assets(&self, rows: &[LostAssetRow]) -> CoreResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        sqlx::query("DELETE FROM lost_asset_balances")
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        let now = now_str();
        for row in rows {
            let lost = row.lost_qty.to_string();
            sqlx::query(
                "INSERT INTO lost_asset_balances
                 (operation_line_id, operation_id, site_id, inventory_subject_id, lost_qty, is_resolved, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&row.operation_line_id).bind(&row.operation_id)
            .bind(0).bind(0).bind(&lost).bind(row.is_resolved).bind(&now)
            .execute(&mut *tx).await.map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        }
        tx.commit()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        Ok(())
    }

    pub async fn write_issued_assets(&self, rows: &[IssuedAssetRow]) -> CoreResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        sqlx::query("DELETE FROM issued_asset_balances")
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        let now = now_str();
        for row in rows {
            let qty = row.qty.to_string();
            sqlx::query(
                "INSERT INTO issued_asset_balances
                 (operation_line_id, operation_id, site_id, inventory_subject_id, qty, issued_to_name, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&row.operation_line_id).bind(&row.operation_id)
            .bind(0).bind(0).bind(&qty).bind(&row.issued_to_name).bind(&now)
            .execute(&mut *tx).await.map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        }
        tx.commit()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        Ok(())
    }

    pub async fn write_stock_summary(
        &self,
        params_hash: &str,
        rows: &[StockSummaryRow],
    ) -> CoreResult<()> {
        let repo = SqliteReportsRepo::new(self.pool.clone());
        repo.cache_stock_summary(params_hash, rows).await
    }
}
