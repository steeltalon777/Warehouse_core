use super::client::{AuthKind, SyncServerClient};
use crate::domain::pagination::PaginatedResponse;
use crate::domain::reports::{ItemMovementRow, StockSummaryRow};
use crate::error::CoreResult;

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
        self.send(req).await
    }
}
