use std::collections::HashMap;

use super::client::{AuthKind, SyncServerClient};
use crate::domain::operation::{
    AcceptLinesRequest, OperationCreate, OperationListItem, OperationResponse, OperationUpdate,
    SetEffectiveAtRequest,
};
use crate::domain::pagination::PaginatedResponse;
use crate::error::{CoreError, CoreResult};

impl SyncServerClient {
    /// GET /operations — list/filter operations (paginated)
    pub async fn operations_list(
        &self,
        page: u32,
        page_size: u32,
        filters: Option<HashMap<&str, &str>>,
    ) -> CoreResult<PaginatedResponse<OperationListItem>> {
        let mut req = self.get("/api/v1/operations", AuthKind::User).query(&[
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

    /// GET /operations/{id} — single operation
    pub async fn operations_get(&self, id: &str) -> CoreResult<OperationResponse> {
        let req = self.get(&format!("/api/v1/operations/{id}"), AuthKind::User);
        self.send(req).await
    }

    /// POST /operations — create new operation
    pub async fn operations_create(&self, op: &OperationCreate) -> CoreResult<OperationResponse> {
        let req = self.post("/api/v1/operations", AuthKind::User).json(op);
        self.send(req).await
    }

    /// POST /operations — from raw JSON payload (for outbox replay)
    ///
    /// If `idempotency_key` is Some, sends `Idempotency-Key` header.
    /// Note: SyncServer POST /operations may not process this header yet.
    /// Full idempotent outbox requires Phase 2 /push protocol (ADR-0008).
    pub async fn create_operation_raw(
        &self,
        payload: &str,
        idempotency_key: Option<&str>,
    ) -> CoreResult<OperationResponse> {
        let v: serde_json::Value =
            serde_json::from_str(payload).map_err(CoreError::Serialization)?;
        let mut req = self.post("/api/v1/operations", AuthKind::User).json(&v);
        if let Some(key) = idempotency_key {
            req = req.header("Idempotency-Key", key);
        }
        self.send(req).await
    }

    /// PATCH /operations/{id} — update operation fields
    pub async fn operations_update(
        &self,
        id: &str,
        update: &OperationUpdate,
    ) -> CoreResult<OperationResponse> {
        let req = self
            .patch(&format!("/api/v1/operations/{id}"), AuthKind::User)
            .json(update);
        self.send(req).await
    }

    /// PATCH /operations/{id}/effective-at — change effective date
    pub async fn operations_set_effective_at(
        &self,
        id: &str,
        effective_at: &str,
    ) -> CoreResult<OperationResponse> {
        let body = SetEffectiveAtRequest {
            effective_at: effective_at.to_string(),
        };
        let req = self
            .patch(
                &format!("/api/v1/operations/{id}/effective-at"),
                AuthKind::User,
            )
            .json(&body);
        self.send(req).await
    }

    /// POST /operations/{id}/submit — submit operation
    pub async fn operations_submit(&self, id: &str) -> CoreResult<OperationResponse> {
        let req = self.post(&format!("/api/v1/operations/{id}/submit"), AuthKind::User);
        self.send(req).await
    }

    /// POST /operations/{id}/cancel — cancel operation
    pub async fn operations_cancel(&self, id: &str) -> CoreResult<OperationResponse> {
        let req = self.post(&format!("/api/v1/operations/{id}/cancel"), AuthKind::User);
        self.send(req).await
    }

    /// POST /operations/{id}/accept-lines — accept/reject lines
    pub async fn operations_accept_lines(
        &self,
        id: &str,
        accept: &AcceptLinesRequest,
    ) -> CoreResult<OperationResponse> {
        let req = self
            .post(
                &format!("/api/v1/operations/{id}/accept-lines"),
                AuthKind::User,
            )
            .json(accept);
        self.send(req).await
    }

    /// DELETE /operations/{id} — delete cancelled operation
    pub async fn operations_delete(&self, id: &str) -> CoreResult<()> {
        let req = self.delete(&format!("/api/v1/operations/{id}"), AuthKind::User);
        self.send_no_body(req).await
    }
}
