use super::client::{AuthKind, SyncServerClient};
use crate::domain::operation::OperationListItem;
use crate::domain::pagination::PaginatedResponse;
use crate::domain::temporary_items::{ApproveAsItemRequest, MergeToItemRequest, TemporaryItemDto};
use crate::error::CoreResult;

impl SyncServerClient {
    /// GET /temporary-items — list temporary items (paginated)
    pub async fn temporary_items_list(
        &self,
        page: u32,
        page_size: u32,
    ) -> CoreResult<PaginatedResponse<TemporaryItemDto>> {
        let req = self.get("/api/v1/temporary-items", AuthKind::User).query(&[
            ("page", page.to_string()),
            ("page_size", page_size.to_string()),
        ]);
        self.send(req).await
    }

    /// GET /temporary-items/{id} — single temporary item
    pub async fn temporary_items_get(&self, id: i32) -> CoreResult<TemporaryItemDto> {
        let req = self.get(&format!("/api/v1/temporary-items/{id}"), AuthKind::User);
        self.send(req).await
    }

    /// POST /temporary-items/{id}/approve-as-item — approve as new catalog item
    pub async fn temporary_items_approve(
        &self,
        id: i32,
        request: &ApproveAsItemRequest,
    ) -> CoreResult<TemporaryItemDto> {
        let req = self
            .post(
                &format!("/api/v1/temporary-items/{id}/approve-as-item"),
                AuthKind::User,
            )
            .json(request);
        self.send(req).await
    }

    /// POST /temporary-items/{id}/merge — merge into existing catalog item
    pub async fn temporary_items_merge(
        &self,
        id: i32,
        request: &MergeToItemRequest,
    ) -> CoreResult<TemporaryItemDto> {
        let req = self
            .post(
                &format!("/api/v1/temporary-items/{id}/merge"),
                AuthKind::User,
            )
            .json(request);
        self.send(req).await
    }

    /// GET /temporary-items/{id}/operations — operations referencing this temp item
    pub async fn temporary_items_operations(
        &self,
        id: i32,
        page: u32,
        page_size: u32,
    ) -> CoreResult<PaginatedResponse<OperationListItem>> {
        let req = self
            .get(
                &format!("/api/v1/temporary-items/{id}/operations"),
                AuthKind::User,
            )
            .query(&[
                ("page", page.to_string()),
                ("page_size", page_size.to_string()),
            ]);
        self.send(req).await
    }

    /// DELETE /temporary-items/{id} — soft-delete a temporary item
    pub async fn temporary_items_delete(&self, id: i32) -> CoreResult<()> {
        let req = self.delete(&format!("/api/v1/temporary-items/{id}"), AuthKind::User);
        self.send_no_body(req).await
    }
}
