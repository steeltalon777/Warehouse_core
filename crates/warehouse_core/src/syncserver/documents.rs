use super::client::{AuthKind, SyncServerClient};
use crate::domain::documents::{DocumentDto, DocumentGenerateRequest};
use crate::domain::pagination::PaginatedResponse;
use crate::error::CoreResult;

impl SyncServerClient {
    /// POST /documents/generate — generate a document from an operation
    pub async fn documents_generate(
        &self,
        request: &DocumentGenerateRequest,
    ) -> CoreResult<DocumentDto> {
        let req = self
            .post("/api/v1/documents/generate", AuthKind::User)
            .json(request);
        self.send(req).await
    }

    /// GET /documents/{id} — document metadata
    pub async fn documents_get(&self, id: uuid::Uuid) -> CoreResult<DocumentDto> {
        let req = self.get(&format!("/api/v1/documents/{id}"), AuthKind::User);
        self.send(req).await
    }

    /// GET /documents/{id}/render — render document (returns raw bytes)
    pub async fn documents_render(&self, id: uuid::Uuid) -> CoreResult<Vec<u8>> {
        let req = self.get(&format!("/api/v1/documents/{id}/render"), AuthKind::User);
        let resp = req
            .send()
            .await
            .map_err(|e| crate::error::CoreError::Network(e.to_string()))?;
        let status = resp.status();
        let body = resp
            .bytes()
            .await
            .map_err(|e| crate::error::CoreError::Network(e.to_string()))?;
        if !status.is_success() {
            let text = String::from_utf8_lossy(&body);
            return Err(crate::error::CoreError::Network(format!(
                "Render failed HTTP {status}: {text}"
            )));
        }
        Ok(body.to_vec())
    }

    /// GET /documents — list documents (paginated, filterable)
    pub async fn documents_list(
        &self,
        offset: u64,
        limit: u64,
    ) -> CoreResult<PaginatedResponse<DocumentDto>> {
        let req = self
            .get("/api/v1/documents", AuthKind::User)
            .query(&[("offset", offset.to_string()), ("limit", limit.to_string())]);
        self.send(req).await
    }

    /// PATCH /documents/{id}/status — update document status
    pub async fn documents_update_status(
        &self,
        id: uuid::Uuid,
        status: &str,
    ) -> CoreResult<DocumentDto> {
        let body = serde_json::json!({"status": status});
        let req = self
            .patch(&format!("/api/v1/documents/{id}/status"), AuthKind::User)
            .json(&body);
        self.send(req).await
    }

    /// GET /documents/operations/{operation_id}/documents — documents for an operation
    pub async fn documents_for_operation(
        &self,
        operation_id: uuid::Uuid,
    ) -> CoreResult<Vec<DocumentDto>> {
        let req = self.get(
            &format!("/api/v1/documents/operations/{operation_id}/documents"),
            AuthKind::User,
        );
        self.send(req).await
    }
}
