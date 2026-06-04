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

/// Wrapper for POST /catalog/admin/categories/bulk response
#[derive(Deserialize)]
struct BulkCategoriesResponse {
    items: Vec<CategoryDto>,
}

/// Wrapper for POST /catalog/admin/units/bulk response
#[derive(Deserialize)]
struct BulkUnitsResponse {
    items: Vec<UnitDto>,
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

    /// POST /catalog/admin/categories/bulk — bulk create categories (admin)
    pub async fn categories_create_bulk(
        &self,
        categories: &[serde_json::Value],
    ) -> CoreResult<Vec<CategoryDto>> {
        let body = serde_json::json!({"items": categories});
        let req = self
            .post("/api/v1/catalog/admin/categories/bulk", AuthKind::User)
            .json(&body);
        let resp: BulkCategoriesResponse = self.send(req).await?;
        Ok(resp.items)
    }

    /// POST /catalog/admin/units/bulk — bulk create units (admin)
    pub async fn units_create_bulk(&self, units: &[serde_json::Value]) -> CoreResult<Vec<UnitDto>> {
        let body = serde_json::json!({"items": units});
        let req = self
            .post("/api/v1/catalog/admin/units/bulk", AuthKind::User)
            .json(&body);
        let resp: BulkUnitsResponse = self.send(req).await?;
        Ok(resp.items)
    }
}
