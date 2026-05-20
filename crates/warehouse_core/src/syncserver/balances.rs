use std::collections::HashMap;

use super::client::{AuthKind, SyncServerClient};
use crate::domain::balance::{BalanceRow, BalanceSummaryRow};
use crate::domain::pagination::PaginatedResponse;
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
struct RawBalanceRow {
    site_id: i32,
    site_name: String,
    inventory_subject_id: i32,
    subject_type: String,
    item_id: Option<i32>,
    temporary_item_id: Option<i32>,
    display_name: String,
    item_name: Option<String>,
    sku: Option<String>,
    unit_symbol: Option<String>,
    qty: serde_json::Value,
    updated_at: String,
}

impl From<RawBalanceRow> for BalanceRow {
    fn from(value: RawBalanceRow) -> Self {
        Self {
            site_id: value.site_id,
            site_code: value.site_name,
            inventory_subject_id: value.inventory_subject_id,
            subject_type: value.subject_type,
            item_id: value.item_id.unwrap_or(value.inventory_subject_id),
            temporary_item_id: value.temporary_item_id,
            item_name: value.item_name.unwrap_or(value.display_name),
            item_sku: value.sku,
            unit_symbol: value.unit_symbol.unwrap_or_default(),
            qty: value.qty,
            updated_at: value.updated_at,
            is_subject: None,
        }
    }
}

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
        let resp: RawPaginatedResponse<RawBalanceRow> = self.send(req).await?;
        Ok(PaginatedResponse {
            items: resp.items.into_iter().map(Into::into).collect(),
            total_count: resp.total_count,
            page: resp.page,
            page_size: resp.page_size,
        })
    }

    /// GET /balances/summary — aggregated balance summary
    pub async fn balances_summary(
        &self,
        page: u32,
        page_size: u32,
    ) -> CoreResult<BalanceSummaryRow> {
        let req = self
            .get("/api/v1/balances/summary", AuthKind::User)
            .query(&[
                ("page", page.to_string()),
                ("page_size", page_size.to_string()),
            ]);
        self.send(req).await
    }
}
