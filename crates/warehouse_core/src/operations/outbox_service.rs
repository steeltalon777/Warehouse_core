use crate::error::{CoreError, CoreResult};
use crate::operations::draft_service::OperationDraftService;
use crate::operations::validation::DraftValidator;
use crate::storage::repos::{DraftRepo, OutboxEvent, OutboxRepo};
use crate::syncserver::SyncServerClient;

pub struct OutboxService<R: OutboxRepo, D: DraftRepo> {
    outbox: R,
    drafts: OperationDraftService<D>,
}

impl<R: OutboxRepo, D: DraftRepo> OutboxService<R, D> {
    pub fn new(outbox: R, drafts: OperationDraftService<D>) -> Self {
        Self { outbox, drafts }
    }

    pub async fn queue_draft_submit(&self, draft_id: &str) -> CoreResult<String> {
        let draft = self
            .drafts
            .get_draft(draft_id)
            .await?
            .ok_or_else(|| CoreError::NotFound(format!("Draft {draft_id}")))?;

        DraftValidator::validate(&draft).map_err(|errors| {
            CoreError::Validation(
                errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; "),
            )
        })?;

        let create = OperationDraftService::<D>::to_operation_create(&draft).ok_or_else(|| {
            CoreError::Validation(
                "Cannot convert draft to operation: missing required fields".to_string(),
            )
        })?;

        let site_id = draft.site_id.unwrap_or(0);
        let payload = serde_json::to_string(&create).map_err(CoreError::Serialization)?;

        let payload_hash = Some(sha256_simple(&payload));
        let idempotency_key = Some(format!("op_create_{}", draft.draft_id));

        self.outbox
            .enqueue(
                "operation_create",
                "create_operation",
                site_id,
                &payload,
                idempotency_key.as_deref(),
                payload_hash.as_deref(),
            )
            .await
    }

    pub async fn list_outbox_events(
        &self,
        site_id: Option<i32>,
        status: Option<&str>,
    ) -> CoreResult<Vec<OutboxEvent>> {
        self.outbox.list(site_id, status).await
    }

    pub async fn get_outbox_event(&self, event_uuid: &str) -> CoreResult<Option<OutboxEvent>> {
        self.outbox.get(event_uuid).await
    }

    pub async fn retry_event(&self, event_uuid: &str) -> CoreResult<()> {
        self.outbox.retry_event(event_uuid).await
    }

    pub async fn cancel_event(&self, event_uuid: &str) -> CoreResult<()> {
        self.outbox.cancel_event(event_uuid).await
    }

    pub async fn count_pending(&self) -> CoreResult<i64> {
        self.outbox.count_pending().await
    }

    pub async fn send_pending(
        &self,
        client: &SyncServerClient,
        max_batch: i32,
    ) -> CoreResult<SendResult> {
        let events = self.outbox.dequeue(max_batch).await?;
        if events.is_empty() {
            return Ok(SendResult::default());
        }

        let mut accepted = 0i64;
        let mut failed = 0i64;
        let mut conflicts = 0i64;

        for ev in &events {
            self.outbox.mark_sending(&ev.event_uuid).await?;

            match client
                .create_operation_raw(&ev.payload, ev.idempotency_key.as_deref())
                .await
            {
                Ok(response) => {
                    let result = serde_json::to_string(&response).unwrap_or_default();
                    self.outbox.mark_success(&ev.event_uuid).await?;
                    self.outbox
                        .update_server_result(&ev.event_uuid, &result)
                        .await?;
                    accepted += 1;
                }
                Err(CoreError::Conflict(_)) | Err(CoreError::Validation(_)) => {
                    self.outbox
                        .mark_conflict(&ev.event_uuid, "Server rejected the command")
                        .await?;
                    conflicts += 1;
                }
                Err(CoreError::Auth(_)) => {
                    self.outbox
                        .mark_failed(&ev.event_uuid, "Auth failed")
                        .await?;
                    failed += 1;
                }
                Err(CoreError::NotFound(_)) => {
                    self.outbox
                        .mark_dead_letter(&ev.event_uuid, "Endpoint not found")
                        .await?;
                    failed += 1;
                }
                Err(_) => {
                    self.outbox
                        .mark_failed(&ev.event_uuid, "Network/server error")
                        .await?;
                    failed += 1;
                }
            }
        }

        Ok(SendResult {
            accepted,
            failed,
            conflicts,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct SendResult {
    pub accepted: i64,
    pub failed: i64,
    pub conflicts: i64,
}

fn sha256_simple(input: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
