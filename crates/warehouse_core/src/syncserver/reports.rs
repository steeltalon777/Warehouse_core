use super::client::{AuthKind, SyncServerClient};
use crate::domain::pagination::PaginatedResponse;
use crate::domain::reports::{ItemMovementRow, StockSummaryRow};
use crate::error::CoreResult;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RawPaginatedResponse<T> {
    items: Vec<T>,
    total_count: u64,
    page: u32,
    page_size: u32,
}

#[derive(Debug, Deserialize)]
struct RawStockSummaryRow {
    site_id: i32,
    site_name: String,
    inventory_subject_id: i32,
    item_id: Option<i32>,
    resolved_item_name: Option<String>,
    display_name: String,
    total_quantity: serde_json::Value,
    last_balance_at: Option<String>,
}

impl From<RawStockSummaryRow> for StockSummaryRow {
    fn from(value: RawStockSummaryRow) -> Self {
        Self {
            item_id: value.item_id.unwrap_or(value.inventory_subject_id),
            item_name: value.resolved_item_name.unwrap_or(value.display_name),
            item_sku: None,
            unit_symbol: String::new(),
            site_id: value.site_id,
            site_code: value.site_name,
            quantity: value.total_quantity,
            last_operation_at: value.last_balance_at,
        }
    }
}

impl SyncServerClient {
    /// GET /reports/item-movement — item movement report
    pub async fn reports_item_movement(
        &self,
        page: u32,
        page_size: u32,
        item_id: Option<i32>,
        date_from: Option<&str>,
        date_to: Option<&str>,
    ) -> CoreResult<PaginatedResponse<ItemMovementRow>> {
        let mut req = self
            .get("/api/v1/reports/item-movement", AuthKind::User)
            .query(&[
                ("page", page.to_string()),
                ("page_size", page_size.to_string()),
            ]);
        if let Some(id) = item_id {
            req = req.query(&[("item_id", id.to_string())]);
        }
        if let Some(f) = date_from {
            req = req.query(&[("date_from", f)]);
        }
        if let Some(t) = date_to {
            req = req.query(&[("date_to", t)]);
        }
        self.send(req).await
    }

    /// GET /reports/stock-summary — stock summary report
    pub async fn reports_stock_summary(
        &self,
        page: u32,
        page_size: u32,
    ) -> CoreResult<PaginatedResponse<StockSummaryRow>> {
        let req = self
            .get("/api/v1/reports/stock-summary", AuthKind::User)
            .query(&[
                ("page", page.to_string()),
                ("page_size", page_size.to_string()),
            ]);
        let resp: RawPaginatedResponse<RawStockSummaryRow> = self.send(req).await?;
        Ok(PaginatedResponse {
            items: resp.items.into_iter().map(Into::into).collect(),
            total_count: resp.total_count,
            page: resp.page,
            page_size: resp.page_size,
        })
    }
}
