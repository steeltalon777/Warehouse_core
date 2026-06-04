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
    #[serde(deserialize_with = "crate::domain::serde_helpers::string_or_number")]
    operation_line_id: String,
    inventory_subject_id: i32,
    item_id: Option<i32>,
    display_name: String,
    item_name: Option<String>,
    sku: Option<String>,
    qty: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct RawIssuedAssetRow {
    issue_object_id: Option<i32>,
    issue_object_name: Option<String>,
    issue_object_type: Option<String>,
    inventory_subject_id: Option<i32>,
    item_id: Option<i32>,
    #[serde(default)]
    display_name: String,
    #[serde(default)]
    item_name: Option<String>,
    #[serde(default)]
    sku: Option<String>,
    qty: serde_json::Value,
}

impl From<RawLostAssetRow> for LostAssetRow {
    fn from(value: RawLostAssetRow) -> Self {
        let qty = value.qty;
        Self {
            operation_id: value.operation_id,
            operation_line_id: value.operation_line_id,
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
        // API returns issue-object-based data (no operation_id/line).
        // Generate a synthetic PK from issue_object_id + inventory_subject_id.
        let inv_id = value.inventory_subject_id.unwrap_or(0);
        let obj_id = value.issue_object_id.unwrap_or(0);
        Self {
            operation_id: format!("issue_obj_{}", obj_id),
            operation_line_id: format!("{}_{}", obj_id, inv_id),
            item_id: value.item_id.unwrap_or(inv_id),
            item_name: value.item_name.unwrap_or(value.display_name),
            item_sku: value.sku,
            unit_symbol: String::new(),
            qty: value.qty,
            issued_to_name: value.issue_object_name.unwrap_or_default(),
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
    pub async fn lost_assets_get(&self, operation_line_id: &str) -> CoreResult<LostAssetRow> {
        let req = self.get(
            &format!("/api/v1/lost-assets/{}", operation_line_id),
            AuthKind::User,
        );
        self.send(req).await
    }

    /// POST /lost-assets/{operation_line_id}/resolve — resolve a lost asset
    pub async fn lost_assets_resolve(
        &self,
        operation_line_id: &str,
        request: &LostAssetResolveRequest,
    ) -> CoreResult<serde_json::Value> {
        let req = self
            .post(
                &format!("/api/v1/lost-assets/{}/resolve", operation_line_id),
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
