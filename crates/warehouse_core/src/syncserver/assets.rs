use std::collections::HashMap;

use super::client::{AuthKind, SyncServerClient};
use crate::domain::assets::{
    IssuedAssetRow, LostAssetResolveRequest, LostAssetRow, PendingAcceptanceRow,
};
use crate::domain::pagination::PaginatedResponse;
use crate::error::CoreResult;

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
        self.send(req).await
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
        self.send(req).await
    }
}
