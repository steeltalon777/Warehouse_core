use crate::domain::operation::{
    OperationCreate, OperationDraft, OperationDraftLine, OperationLineCreate, OperationType,
    TemporaryItemInlineCreate,
};
use crate::error::CoreResult;
use crate::storage::repos::DraftRepo;
use uuid::Uuid;

pub struct OperationDraftService<R: DraftRepo> {
    repo: R,
}

impl<R: DraftRepo> OperationDraftService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_draft(
        &self,
        operation_type: OperationType,
        site_id: Option<i32>,
    ) -> CoreResult<OperationDraft> {
        let now = crate::time::Timestamp::now_utc().to_string();
        let draft = OperationDraft {
            draft_id: Uuid::new_v4(),
            operation_type,
            site_id,
            lines: Vec::new(),
            effective_at: None,
            source_site_id: None,
            destination_site_id: None,
            recipient_id: None,
            issued_to_name: None,
            comment: None,
            created_at: now.clone(),
            updated_at: now,
        };
        self.repo.save(&draft).await?;
        Ok(draft)
    }

    pub async fn get_draft(&self, draft_id: &str) -> CoreResult<Option<OperationDraft>> {
        self.repo.get(draft_id).await
    }

    pub async fn list_drafts(&self) -> CoreResult<Vec<OperationDraft>> {
        self.repo.list().await
    }

    pub async fn delete_draft(&self, draft_id: &str) -> CoreResult<()> {
        self.repo.delete(draft_id).await
    }

    pub async fn clone_draft(&self, draft_id: &str) -> CoreResult<Option<OperationDraft>> {
        let original = match self.repo.get(draft_id).await? {
            Some(d) => d,
            None => return Ok(None),
        };
        let now = crate::time::Timestamp::now_utc().to_string();
        let cloned = OperationDraft {
            draft_id: Uuid::new_v4(),
            lines: original
                .lines
                .into_iter()
                .map(|l| OperationDraftLine {
                    line_id: Uuid::new_v4(),
                    ..l
                })
                .collect(),
            created_at: now.clone(),
            updated_at: now,
            ..original
        };
        self.repo.save(&cloned).await?;
        Ok(Some(cloned))
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_header(
        &self,
        draft_id: &str,
        operation_type: Option<OperationType>,
        site_id: Option<Option<i32>>,
        effective_at: Option<Option<String>>,
        source_site_id: Option<Option<i32>>,
        destination_site_id: Option<Option<i32>>,
        recipient_id: Option<Option<i32>>,
        issued_to_name: Option<Option<String>>,
        comment: Option<Option<String>>,
    ) -> CoreResult<Option<OperationDraft>> {
        let mut draft = match self.repo.get(draft_id).await? {
            Some(d) => d,
            None => return Ok(None),
        };
        if let Some(op) = operation_type {
            draft.operation_type = op;
        }
        if let Some(s) = site_id {
            draft.site_id = s;
        }
        if let Some(e) = effective_at {
            draft.effective_at = e;
        }
        if let Some(s) = source_site_id {
            draft.source_site_id = s;
        }
        if let Some(d) = destination_site_id {
            draft.destination_site_id = d;
        }
        if let Some(r) = recipient_id {
            draft.recipient_id = r;
        }
        if let Some(i) = issued_to_name {
            draft.issued_to_name = i;
        }
        if let Some(c) = comment {
            draft.comment = c;
        }
        draft.updated_at = crate::time::Timestamp::now_utc().to_string();
        self.repo.save(&draft).await?;
        Ok(Some(draft))
    }

    pub async fn add_item_line(
        &self,
        draft_id: &str,
        item_id: i32,
        qty: serde_json::Value,
        batch: Option<String>,
        comment: Option<String>,
    ) -> CoreResult<Option<OperationDraft>> {
        let mut draft = match self.repo.get(draft_id).await? {
            Some(d) => d,
            None => return Ok(None),
        };
        draft.lines.push(OperationDraftLine {
            line_id: Uuid::new_v4(),
            item_id: Some(item_id),
            temporary_item: None,
            qty,
            batch,
            comment,
        });
        draft.updated_at = crate::time::Timestamp::now_utc().to_string();
        self.repo.save(&draft).await?;
        Ok(Some(draft))
    }

    pub async fn add_temp_item_line(
        &self,
        draft_id: &str,
        temp_item: TemporaryItemInlineCreate,
        qty: serde_json::Value,
        batch: Option<String>,
        comment: Option<String>,
    ) -> CoreResult<Option<OperationDraft>> {
        let mut draft = match self.repo.get(draft_id).await? {
            Some(d) => d,
            None => return Ok(None),
        };
        draft.lines.push(OperationDraftLine {
            line_id: Uuid::new_v4(),
            item_id: None,
            temporary_item: Some(temp_item),
            qty,
            batch,
            comment,
        });
        draft.updated_at = crate::time::Timestamp::now_utc().to_string();
        self.repo.save(&draft).await?;
        Ok(Some(draft))
    }

    pub async fn update_line(
        &self,
        draft_id: &str,
        line_id: &str,
        qty: Option<serde_json::Value>,
        batch: Option<Option<String>>,
        comment: Option<Option<String>>,
    ) -> CoreResult<Option<OperationDraft>> {
        let mut draft = match self.repo.get(draft_id).await? {
            Some(d) => d,
            None => return Ok(None),
        };
        let line_uuid = match Uuid::parse_str(line_id) {
            Ok(u) => u,
            Err(_) => return Ok(None),
        };
        if let Some(line) = draft.lines.iter_mut().find(|l| l.line_id == line_uuid) {
            if let Some(q) = qty {
                line.qty = q;
            }
            if let Some(b) = batch {
                line.batch = b;
            }
            if let Some(c) = comment {
                line.comment = c;
            }
            draft.updated_at = crate::time::Timestamp::now_utc().to_string();
            self.repo.save(&draft).await?;
            Ok(Some(draft))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_line(
        &self,
        draft_id: &str,
        line_id: &str,
    ) -> CoreResult<Option<OperationDraft>> {
        let mut draft = match self.repo.get(draft_id).await? {
            Some(d) => d,
            None => return Ok(None),
        };
        let line_uuid = match Uuid::parse_str(line_id) {
            Ok(u) => u,
            Err(_) => return Ok(None),
        };
        let before = draft.lines.len();
        draft.lines.retain(|l| l.line_id != line_uuid);
        if draft.lines.len() < before {
            draft.updated_at = crate::time::Timestamp::now_utc().to_string();
            self.repo.save(&draft).await?;
            Ok(Some(draft))
        } else {
            Ok(None)
        }
    }

    pub fn to_operation_create(draft: &OperationDraft) -> Option<OperationCreate> {
        let site_id = draft.site_id?;
        if draft.lines.is_empty() {
            return None;
        }
        let lines: Vec<OperationLineCreate> = draft
            .lines
            .iter()
            .filter_map(|l| {
                let item_id = l.item_id?;
                Some(OperationLineCreate {
                    item_id,
                    qty: l.qty.clone(),
                    batch: l.batch.clone(),
                    comment: l.comment.clone(),
                })
            })
            .collect();
        if lines.is_empty() {
            return None;
        }
        Some(OperationCreate {
            operation_type: draft.operation_type.clone(),
            site_id,
            lines,
            effective_at: draft.effective_at.clone(),
            source_site_id: draft.source_site_id,
            destination_site_id: draft.destination_site_id,
            recipient_id: draft.recipient_id,
            issued_to_name: draft.issued_to_name.clone(),
            comment: draft.comment.clone(),
        })
    }
}
