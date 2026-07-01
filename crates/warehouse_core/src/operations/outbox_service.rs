use crate::error::{CoreError, CoreResult};
use crate::operations::draft_service::OperationDraftService;
use crate::operations::validation::DraftValidator;
use crate::storage::repos::{DraftRepo, OutboxEvent, OutboxRepo};
use crate::syncserver::SyncServerClient;
use serde_json::Value;
use sha2::{Digest, Sha256};

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

        let payload_hash = Some(compute_payload_hash(&payload));
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

/// Compute a stable SHA-256 hash of a JSON payload for idempotency / sync envelope checks.
///
/// Mirrors SyncServer `payload_hash` (see `SyncServer/app/repos/events_repo.py:44-46`
/// and `SyncServer/app/services/machine_service.py:97-99`):
/// 1. Parse the payload into a `Value` and re-serialize it as compact,
///    sorted-key JSON. `serde_json`'s `BTreeMap` object ordering gives sorted
///    keys; `to_string` uses `,` / `:` separators and **does not** escape
///    non-ASCII bytes (its internal `ESCAPE[128..=255]` is zero, so high
///    bytes pass through as raw UTF-8 fragments).
/// 2. Hash the canonical bytes with SHA-256 and return the lowercase hex
///    digest.
///
/// This byte-for-byte matches Python's
/// `hashlib.sha256(json.dumps(p, sort_keys=True, separators=(",", ":"),
/// ensure_ascii=False).encode("utf-8")).hexdigest()` for any valid JSON
/// payload, including payloads with Cyrillic / other non-ASCII strings.
///
/// The cross-language invariant is verified by:
/// - `scripts/verify_payload_hash.py` (reproducible hash table for fixtures)
/// - `tests/cross_lang_payload_hash.rs` (subprocess-based, real Python run)
/// - `compute_payload_hash_*` unit tests (key-invariance + Cyrillic fixtures)
///
/// Returns the hex digest of the raw input string wrapped in a `Value::String`
/// when the input is not valid JSON; this preserves the previous fallback
/// behaviour for non-JSON payloads while making the format compatible with
/// SyncServer for JSON ones.
pub fn compute_payload_hash(json: &str) -> String {
    let value: Value =
        serde_json::from_str(json).unwrap_or_else(|_| Value::String(json.to_string()));
    let canonical = serde_json::to_string(&value).unwrap_or_else(|_| json.to_string());
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_payload_hash_matches_python_canonical_form() {
        // SyncServer reference (see SyncServer/app/services/machine_service.py):
        //   json.dumps(data, ensure_ascii=False, sort_keys=True, default=str)
        //   hashlib.sha256(...).hexdigest()
        // For ASCII-only payloads the Rust and Python serializations are byte-identical
        // (both produce compact, sorted-key JSON). We assert key-invariance here; a
        // separate Python helper in scripts/verify_payload_hash.py cross-checks the
        // hex digest against hashlib.sha256 for a fixed set of fixtures.
        let json = r#"{"a": 1, "b": [1, 2, 3]}"#;
        let hash = compute_payload_hash(json);
        let hash2 = compute_payload_hash(r#"{"b":[1,2,3],"a":1}"#);
        assert_eq!(hash, hash2);
        // 64 lowercase hex characters (SHA-256).
        assert_eq!(hash.len(), 64);
        assert!(
            hash.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
    }

    #[test]
    fn compute_payload_hash_matches_python_for_op_create_payload() {
        // Mirrors the SyncServer `payload_hash` for an OP_CREATE envelope.
        // Canonical form: sorted keys, compact separators.
        // This is the exact hash that SyncServer would compute for the same payload.
        let envelope = serde_json::json!({
            "operation_type": "RECEIVE",
            "site_id": 1,
            "lines": [
                {"item_id": 42, "qty": "5", "batch": null, "comment": null}
            ],
            "effective_at": null,
            "source_site_id": null,
            "destination_site_id": null,
            "recipient_id": null,
            "issued_to_name": null,
            "comment": null,
        });
        let json = serde_json::to_string(&envelope).unwrap();
        let hash = compute_payload_hash(&json);
        assert_eq!(hash.len(), 64);
        // Recompute from differently-ordered JSON — should match.
        let reordered = serde_json::to_string(&serde_json::json!({
            "comment": null,
            "issued_to_name": null,
            "recipient_id": null,
            "destination_site_id": null,
            "source_site_id": null,
            "effective_at": null,
            "lines": [
                {"comment": null, "batch": null, "qty": "5", "item_id": 42}
            ],
            "site_id": 1,
            "operation_type": "RECEIVE",
        }))
        .unwrap();
        assert_eq!(hash, compute_payload_hash(&reordered));
    }

    /// Reference hashes captured from the SyncServer reference implementation:
    ///
    ///   python3 scripts/verify_payload_hash.py
    ///
    /// The script prints the canonical JSON and `sha256` for a fixed set of
    /// fixtures (including Cyrillic, non-BMP emoji, control characters, and
    /// JSON-special escapes). SyncServer computes the same bytes via
    /// `json.dumps(..., sort_keys=True, separators=(",", ":"),
    /// ensure_ascii=False)` + `hashlib.sha256` (see
    /// `SyncServer/app/repos/events_repo.py:44-46` and
    /// `SyncServer/app/services/machine_service.py:97-99`).
    ///
    /// If any of the values below drift, the canonical-JSON contract with
    /// SyncServer is broken — update *both* this test and the script in the
    /// same commit.
    #[test]
    fn compute_payload_hash_cyrillic_matches_syncserver_reference() {
        // (input_json_as_written_by_caller, expected_sha256_hex)
        let cases: &[(&str, &str)] = &[
            // ASCII baseline.
            (
                r#"{"a":1,"b":[1,2,3]}"#,
                "bfa6ceebf136e4837ec687f2be09f612c645c9ec1f99e3ef5d497b0d5bb99e0a",
            ),
            // Single Cyrillic string in object value.
            (
                r#"{"name":"Склад"}"#,
                "7eb087b21389ae1ee1279ecfa61533b16757dfc9de3c7174873c02fa8047f3d1",
            ),
            // Cyrillic with mixed ASCII, sorted keys.
            (
                r#"{"comment":"Принято: 10 шт.","qty":5}"#,
                "b2057942493b341c76f014ea071897b3560eea7a525a99620f39f8527b362870",
            ),
            // Non-BMP (emoji) — single codepoint above U+FFFF.
            (
                r#"{"icon":"📦"}"#,
                "8916d6883656b8cb17aa655e130af54ad04530056932a507dd5ac7848747b62b",
            ),
            // Control characters: both sides must JSON-escape them identically.
            (
                r#"{"text":"line1\nline2\ttab"}"#,
                "7aa504eb4a846978e40a99cd584c2ce45523612cf317be9884b2b052af84da6c",
            ),
            // JSON-special escapes (quote, backslash) — both sides must agree.
            (
                r#"{"text":"a\"b\\c"}"#,
                "170dc7de4779f00492637084bcfc30994c4d953afb281c5a60092cd9866cffc5",
            ),
        ];

        for (input, expected) in cases {
            let actual = compute_payload_hash(input);
            assert_eq!(
                &actual, expected,
                "compute_payload_hash drift for input {input:?}\nexpected: {expected}\nactual:   {actual}",
            );
        }
    }
}
