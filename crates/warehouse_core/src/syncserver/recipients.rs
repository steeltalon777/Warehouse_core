use super::client::{AuthKind, SyncServerClient};
use crate::domain::pagination::PaginatedResponse;
use crate::domain::recipient::{RecipientCreate, RecipientDto};
use crate::error::CoreResult;

impl SyncServerClient {
    /// GET /recipients — list recipients (paginated, searchable)
    pub async fn recipients_list(
        &self,
        page: u32,
        page_size: u32,
        search: Option<&str>,
    ) -> CoreResult<PaginatedResponse<RecipientDto>> {
        let mut req = self.get("/api/v1/recipients", AuthKind::User).query(&[
            ("page", page.to_string()),
            ("page_size", page_size.to_string()),
        ]);
        if let Some(q) = search {
            req = req.query(&[("search", q)]);
        }
        self.send(req).await
    }

    /// POST /recipients — create a recipient
    pub async fn recipients_create(&self, recipient: &RecipientCreate) -> CoreResult<RecipientDto> {
        let req = self
            .post("/api/v1/recipients", AuthKind::User)
            .json(recipient);
        self.send(req).await
    }

    /// POST /recipients/merge — merge two recipients
    pub async fn recipients_merge(
        &self,
        source_id: i32,
        target_id: i32,
    ) -> CoreResult<RecipientDto> {
        let body = serde_json::json!({"source_id": source_id, "target_id": target_id});
        let req = self
            .post("/api/v1/recipients/merge", AuthKind::User)
            .json(&body);
        self.send(req).await
    }

    /// GET /recipients/{id} — single recipient
    pub async fn recipients_get(&self, id: i32) -> CoreResult<RecipientDto> {
        let req = self.get(&format!("/api/v1/recipients/{id}"), AuthKind::User);
        self.send(req).await
    }

    /// PATCH /recipients/{id} — update a recipient
    pub async fn recipients_update(
        &self,
        id: i32,
        update: &serde_json::Value,
    ) -> CoreResult<RecipientDto> {
        let req = self
            .patch(&format!("/api/v1/recipients/{id}"), AuthKind::User)
            .json(update);
        self.send(req).await
    }

    /// DELETE /recipients/{id} — delete a recipient
    pub async fn recipients_delete(&self, id: i32) -> CoreResult<()> {
        let req = self.delete(&format!("/api/v1/recipients/{id}"), AuthKind::User);
        self.send_no_body(req).await
    }
}
