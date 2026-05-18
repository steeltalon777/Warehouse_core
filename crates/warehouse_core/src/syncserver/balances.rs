use std::collections::HashMap;

use super::client::{AuthKind, SyncServerClient};
use crate::domain::balance::{BalanceRow, BalanceSummaryRow};
use crate::domain::pagination::PaginatedResponse;
use crate::error::CoreResult;

impl SyncServerClient {
    // ── Balances ────────────────────────────────────────

    /// GET /balances — list balances (filterable)
    pub async fn balances_list(
        &self,
        page: u32,
        page_size: u32,
        filters: Option<HashMap<&str, &str>>,
    ) -> CoreResult<PaginatedResponse<BalanceRow>> {
        let mut req = self.get("/api/v1/balances", AuthKind::User).query(&[
            ("page", page.to_string()),
            ("page_size", page_size.to_string()),
        ]);
        if let Some(f) = filters {
            for (k, v) in f {
                req = req.query(&[(k, v)]);
            }
        }
        self.send(req).await
    }

    /// GET /balances/by-site — balances for specific site
    pub async fn balances_by_site(
        &self,
        site_id: i32,
        page: u32,
        page_size: u32,
    ) -> CoreResult<PaginatedResponse<BalanceRow>> {
        let req = self
            .get("/api/v1/balances/by-site", AuthKind::User)
            .query(&[
                ("site_id", site_id.to_string()),
                ("page", page.to_string()),
                ("page_size", page_size.to_string()),
            ]);
        self.send(req).await
    }

    /// GET /balances/summary — aggregated balance summary
    pub async fn balances_summary(
        &self,
        page: u32,
        page_size: u32,
    ) -> CoreResult<PaginatedResponse<BalanceSummaryRow>> {
        let req = self
            .get("/api/v1/balances/summary", AuthKind::User)
            .query(&[
                ("page", page.to_string()),
                ("page_size", page_size.to_string()),
            ]);
        self.send(req).await
    }
}
