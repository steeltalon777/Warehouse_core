use serde::Deserialize;

use super::client::{AuthKind, SyncServerClient};
use crate::domain::catalog::{CatalogSiteDto, CategoryDto, CategoryTreeNode, ItemDto, UnitDto};
use crate::domain::pagination::CursorResponse;
use crate::error::CoreResult;

/// Wrapper for /catalog/sites response
#[derive(Deserialize)]
struct CatalogSitesResponse {
    sites: Vec<CatalogSiteDto>,
    #[allow(dead_code)]
    server_time: String,
}

impl SyncServerClient {
    pub async fn catalog_items(
        &self,
        updated_after: Option<&str>,
        limit: Option<u32>,
    ) -> CoreResult<CursorResponse<ItemDto>> {
        let mut req = self.get("/api/v1/catalog/items", AuthKind::User);
        if let Some(after) = updated_after {
            req = req.query(&[("updated_after", after)]);
        }
        if let Some(l) = limit {
            req = req.query(&[("limit", l.to_string())]);
        }
        self.send(req).await
    }

    pub async fn catalog_categories(
        &self,
        updated_after: Option<&str>,
        limit: Option<u32>,
    ) -> CoreResult<CursorResponse<CategoryDto>> {
        let mut req = self.get("/api/v1/catalog/categories", AuthKind::User);
        if let Some(after) = updated_after {
            req = req.query(&[("updated_after", after)]);
        }
        if let Some(l) = limit {
            req = req.query(&[("limit", l.to_string())]);
        }
        self.send(req).await
    }

    pub async fn catalog_category_tree(&self) -> CoreResult<Vec<CategoryTreeNode>> {
        let req = self.get("/api/v1/catalog/categories/tree", AuthKind::User);
        self.send(req).await
    }

    pub async fn catalog_units(
        &self,
        updated_after: Option<&str>,
        limit: Option<u32>,
    ) -> CoreResult<CursorResponse<UnitDto>> {
        let mut req = self.get("/api/v1/catalog/units", AuthKind::User);
        if let Some(after) = updated_after {
            req = req.query(&[("updated_after", after)]);
        }
        if let Some(l) = limit {
            req = req.query(&[("limit", l.to_string())]);
        }
        self.send(req).await
    }

    pub async fn catalog_sites(&self) -> CoreResult<Vec<CatalogSiteDto>> {
        let req = self.get("/api/v1/catalog/sites", AuthKind::User);
        let resp: CatalogSitesResponse = self.send(req).await?;
        Ok(resp.sites)
    }
}
