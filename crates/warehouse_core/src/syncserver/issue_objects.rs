use super::client::{AuthKind, SyncServerClient};
use crate::domain::issue_objects::{
    IssueObjectCategoryCreate, IssueObjectCategoryDto, IssueObjectCategoryUpdate,
    IssueObjectCreate, IssueObjectDto, IssueObjectListResponse, IssueObjectMerge,
    IssueObjectTreeDto, IssueObjectUpdate,
};
use crate::domain::pagination::PaginatedResponse;
use crate::error::CoreResult;

impl SyncServerClient {
    /// GET /api/v1/issue-objects — list issue objects (paginated, searchable)
    pub async fn issue_objects_list(
        &self,
        page: u32,
        page_size: u32,
        search: Option<&str>,
    ) -> CoreResult<IssueObjectListResponse> {
        let mut req = self.get("/api/v1/issue-objects", AuthKind::User).query(&[
            ("page", page.to_string()),
            ("page_size", page_size.to_string()),
        ]);
        if let Some(q) = search {
            req = req.query(&[("search", q)]);
        }
        self.send(req).await
    }

    /// GET /api/v1/issue-objects/{id} — single issue object
    pub async fn issue_objects_get(&self, id: i32) -> CoreResult<IssueObjectDto> {
        let req = self.get(&format!("/api/v1/issue-objects/{id}"), AuthKind::User);
        self.send(req).await
    }

    /// POST /api/v1/issue-objects — create an issue object
    pub async fn issue_objects_create(
        &self,
        body: &IssueObjectCreate,
    ) -> CoreResult<IssueObjectDto> {
        let req = self
            .post("/api/v1/issue-objects", AuthKind::User)
            .json(body);
        self.send(req).await
    }

    /// PATCH /api/v1/issue-objects/{id} — update an issue object
    pub async fn issue_objects_update(
        &self,
        id: i32,
        body: &IssueObjectUpdate,
    ) -> CoreResult<IssueObjectDto> {
        let req = self
            .patch(&format!("/api/v1/issue-objects/{id}"), AuthKind::User)
            .json(body);
        self.send(req).await
    }

    /// DELETE /api/v1/issue-objects/{id} — delete an issue object
    pub async fn issue_objects_delete(&self, id: i32) -> CoreResult<()> {
        let req = self.delete(&format!("/api/v1/issue-objects/{id}"), AuthKind::User);
        self.send_no_body(req).await
    }

    /// POST /api/v1/issue-objects/merge — merge two issue objects
    pub async fn issue_objects_merge(&self, body: &IssueObjectMerge) -> CoreResult<IssueObjectDto> {
        let req = self
            .post("/api/v1/issue-objects/merge", AuthKind::User)
            .json(body);
        self.send(req).await
    }

    /// GET /api/v1/issue-objects/{id}/assets — list assets for an issue object
    pub async fn issue_objects_list_assets(
        &self,
        id: i32,
        page: u32,
        page_size: u32,
    ) -> CoreResult<PaginatedResponse<serde_json::Value>> {
        let req = self
            .get(
                &format!("/api/v1/issue-objects/{id}/assets"),
                AuthKind::User,
            )
            .query(&[
                ("page", page.to_string()),
                ("page_size", page_size.to_string()),
            ]);
        self.send(req).await
    }

    /// GET /api/v1/issue-objects/tree — full issue object tree (categories + objects)
    pub async fn issue_objects_get_tree(&self) -> CoreResult<Vec<IssueObjectTreeDto>> {
        let req = self.get("/api/v1/issue-objects/tree", AuthKind::User);
        self.send(req).await
    }

    /// POST /api/v1/issue-object-categories — create a category
    pub async fn issue_object_categories_create(
        &self,
        body: &IssueObjectCategoryCreate,
    ) -> CoreResult<IssueObjectCategoryDto> {
        let req = self
            .post("/api/v1/issue-object-categories", AuthKind::User)
            .json(body);
        self.send(req).await
    }

    /// GET /api/v1/issue-object-categories — list categories (paginated)
    pub async fn issue_object_categories_list(
        &self,
        page: u32,
        page_size: u32,
    ) -> CoreResult<PaginatedResponse<IssueObjectCategoryDto>> {
        let req = self
            .get("/api/v1/issue-object-categories", AuthKind::User)
            .query(&[
                ("page", page.to_string()),
                ("page_size", page_size.to_string()),
            ]);
        self.send(req).await
    }

    /// GET /api/v1/issue-object-categories/{id} — single category
    pub async fn issue_object_categories_get(&self, id: i32) -> CoreResult<IssueObjectCategoryDto> {
        let req = self.get(
            &format!("/api/v1/issue-object-categories/{id}"),
            AuthKind::User,
        );
        self.send(req).await
    }

    /// PATCH /api/v1/issue-object-categories/{id} — update a category
    pub async fn issue_object_categories_update(
        &self,
        id: i32,
        body: &IssueObjectCategoryUpdate,
    ) -> CoreResult<IssueObjectCategoryDto> {
        let req = self
            .patch(
                &format!("/api/v1/issue-object-categories/{id}"),
                AuthKind::User,
            )
            .json(body);
        self.send(req).await
    }

    /// DELETE /api/v1/issue-object-categories/{id} — delete a category
    pub async fn issue_object_categories_delete(&self, id: i32) -> CoreResult<()> {
        let req = self.delete(
            &format!("/api/v1/issue-object-categories/{id}"),
            AuthKind::User,
        );
        self.send_no_body(req).await
    }
}
