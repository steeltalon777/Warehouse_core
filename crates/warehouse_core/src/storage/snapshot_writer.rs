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
/// Each method clears the family's local data and replaces it atomically.
pub struct SnapshotWriter {
    pool: SqlitePool,
}

impl SnapshotWriter {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn write_items(&self, items: &[ItemDto]) -> CoreResult<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        sqlx::query("DELETE FROM items")
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        sqlx::query("PRAGMA foreign_keys = OFF")
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        for item in items {
            let hashtags = item
                .hashtags
                .as_ref()
                .map(|h| serde_json::to_string(h).unwrap_or_default());
            let created_at = item.created_at.as_deref().unwrap_or(&item.updated_at);
            sqlx::query(
                "INSERT INTO items (id, sku, name, category_id, unit_id, description, is_active, hashtags, updated_at, created_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(item.id).bind(&item.sku).bind(&item.name)
            .bind(item.category_id).bind(item.unit_id).bind(&item.description)
            .bind(item.is_active).bind(&hashtags).bind(&item.updated_at)
            .bind(created_at)
            .execute(&mut *tx).await.map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        }
        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
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
        sqlx::query("DELETE FROM categories")
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        let now = now_str();
        for cat in cats {
            sqlx::query(
                "INSERT INTO categories (id, name, parent_id, is_active, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(cat.id).bind(&cat.name).bind(cat.parent_id).bind(cat.is_active).bind(&now).bind(&cat.updated_at)
            .execute(&mut *tx).await.map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
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
        sqlx::query("DELETE FROM units")
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        let now = now_str();
        for unit in units {
            sqlx::query(
                "INSERT INTO units (id, name, symbol, is_active, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(unit.id).bind(&unit.name).bind(&unit.symbol).bind(unit.is_active).bind(&now).bind(&unit.updated_at)
            .execute(&mut *tx).await.map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
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
            sqlx::query("INSERT INTO sites (site_id, code, name, is_active) VALUES (?, ?, ?, ?)")
                .bind(s.site_id)
                .bind(&s.code)
                .bind(&s.name)
                .bind(s.is_active)
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
            let inv_subject_id = r.item_id as i64;
            let qty_str = r.qty.to_string();
            sqlx::query(
                "INSERT INTO balances (site_id, inventory_subject_id, qty, updated_at) VALUES (?, ?, ?, ?)",
            )
            .bind(site_id).bind(inv_subject_id).bind(&qty_str).bind(&now)
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
        if ops.is_empty() {
            return Ok(());
        }
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        sqlx::query("DELETE FROM operation_drafts")
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        let now = now_str();
        for op in ops {
            let op_type = serde_json::to_string(&op.operation_type).unwrap_or_default();
            let _op_status = serde_json::to_string(&op.status).unwrap_or_default();
            sqlx::query(
                "INSERT INTO operation_drafts
                 (draft_id, operation_type, site_id, created_at, updated_at, comment)
                 VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(op.id.to_string())
            .bind(&op_type)
            .bind(op.site_id)
            .bind(&op.created_at)
            .bind(&now)
            .bind(&op.created_by_user_name)
            .execute(&mut *tx)
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
        }
        tx.commit()
            .await
            .map_err(|e| crate::error::CoreError::Database(format!("{e}")))?;
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
            .bind(row.operation_id).bind(row.operation_line_id)
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
            .bind(row.operation_line_id).bind(row.operation_id)
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
            .bind(row.operation_line_id).bind(row.operation_id)
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
