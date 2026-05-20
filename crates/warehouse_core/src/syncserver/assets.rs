use std::collections::HashMap;

use super::client::{AuthKind, SyncServerClient};
use crate::domain::assets::{
    IssuedAssetRow, LostAssetResolveRequest, LostAssetRow, PendingAcceptanceRow,
};
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
struct RawLostAssetRow {
    operation_id: String,
    operation_line_id: i64,
    inventory_subject_id: i32,
    item_id: Option<i32>,
    display_name: String,
    item_name: Option<String>,
    sku: Option<String>,
    qty: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct RawIssuedAssetRow {
    recipient_name: String,
    inventory_subject_id: i32,
    item_id: Option<i32>,
    display_name: String,
    item_name: Option<String>,
    sku: Option<String>,
    qty: serde_json::Value,
}

impl From<RawLostAssetRow> for LostAssetRow {
    fn from(value: RawLostAssetRow) -> Self {
        let qty = value.qty;
        Self {
            operation_id: value.operation_id,
            operation_line_id: value.operation_line_id.to_string(),
            item_id: value.item_id.unwrap_or(value.inventory_subject_id),
            item_name: value.item_name.unwrap_or(value.display_name),
            item_sku: value.sku,
            unit_symbol: String::new(),
            qty: qty.clone(),
            lost_qty: qty,
            is_resolved: false,
        }
    }
}

impl From<RawIssuedAssetRow> for IssuedAssetRow {
    fn from(value: RawIssuedAssetRow) -> Self {
        Self {
            operation_id: String::new(),
            operation_line_id: String::new(),
            item_id: value.item_id.unwrap_or(value.inventory_subject_id),
            item_name: value.item_name.unwrap_or(value.display_name),
            item_sku: value.sku,
            unit_symbol: String::new(),
            qty: value.qty,
            issued_to_name: value.recipient_name,
        }
    }
}

impl SyncServerClient {
    /// GET /pending-acceptance — items pending acceptance (paginated)
    pub async fn pending_acceptance_list(
        &self,
        page: u32,
        page_size: u32,
    ) -> CoreResult<PaginatedResponse<PendingAcceptanceRow>> {
        let req = self
            .get("/api/v1/pending-acceptance", AuthKind::User)
            .query(&[
                ("page", page.to_string()),
                ("page_size", page_size.to_string()),
            ]);
        self.send(req).await
    }

    /// GET /lost-assets — list lost assets (paginated, filterable)
    pub async fn lost_assets_list(
        &self,
        page: u32,
        page_size: u32,
        filters: Option<HashMap<&str, &str>>,
    ) -> CoreResult<PaginatedResponse<LostAssetRow>> {
        let mut req = self.get("/api/v1/lost-assets", AuthKind::User).query(&[
            ("page", page.to_string()),
            ("page_size", page_size.to_string()),
        ]);
        if let Some(f) = filters {
            for (k, v) in f {
                req = req.query(&[(k, v)]);
            }
        }
        let resp: RawPaginatedResponse<RawLostAssetRow> = self.send(req).await?;
        Ok(PaginatedResponse {
            items: resp.items.into_iter().map(Into::into).collect(),
            total_count: resp.total_count,
            page: resp.page,
            page_size: resp.page_size,
        })
    }

    /// GET /lost-assets/{operation_line_id} — single lost asset row
    pub async fn lost_assets_get(&self, operation_line_id: i64) -> CoreResult<LostAssetRow> {
        let req = self.get(
            &format!("/api/v1/lost-assets/{operation_line_id}"),
            AuthKind::User,
        );
        self.send(req).await
    }

    /// POST /lost-assets/{operation_line_id}/resolve — resolve a lost asset
    pub async fn lost_assets_resolve(
        &self,
        operation_line_id: i64,
        request: &LostAssetResolveRequest,
    ) -> CoreResult<serde_json::Value> {
        let req = self
            .post(
                &format!("/api/v1/lost-assets/{operation_line_id}/resolve"),
                AuthKind::User,
            )
            .json(request);
        self.send(req).await
    }

    /// GET /issued-assets — list issued assets (paginated, filterable)
    pub async fn issued_assets_list(
        &self,
        page: u32,
        page_size: u32,
        filters: Option<HashMap<&str, &str>>,
    ) -> CoreResult<PaginatedResponse<IssuedAssetRow>> {
        let mut req = self.get("/api/v1/issued-assets", AuthKind::User).query(&[
            ("page", page.to_string()),
            ("page_size", page_size.to_string()),
        ]);
        if let Some(f) = filters {
            for (k, v) in f {
                req = req.query(&[(k, v)]);
            }
        }
        let resp: RawPaginatedResponse<RawIssuedAssetRow> = self.send(req).await?;
        Ok(PaginatedResponse {
            items: resp.items.into_iter().map(Into::into).collect(),
            total_count: resp.total_count,
            page: resp.page,
            page_size: resp.page_size,
        })
    }
}
