use crate::domain::operation::{OperationDraft, OperationType};

#[derive(Debug, Clone)]
pub struct DraftValidationError {
    pub field: String,
    pub message: String,
}

impl std::fmt::Display for DraftValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

pub struct DraftValidator;

impl DraftValidator {
    pub fn validate(draft: &OperationDraft) -> Result<(), Vec<DraftValidationError>> {
        let mut errors = Vec::new();

        if draft.site_id.is_none() {
            errors.push(DraftValidationError {
                field: "site_id".to_string(),
                message: "Site is required".to_string(),
            });
        }

        if draft.lines.is_empty() {
            errors.push(DraftValidationError {
                field: "lines".to_string(),
                message: "At least one line is required".to_string(),
            });
        }

        for (i, line) in draft.lines.iter().enumerate() {
            if line.item_id.is_none() && line.temporary_item.is_none() {
                errors.push(DraftValidationError {
                    field: format!("lines[{}].item", i),
                    message: "Either item_id or temporary item is required".to_string(),
                });
            }
            match &line.qty {
                serde_json::Value::Number(n) => {
                    if let Some(f) = n.as_f64() {
                        if f <= 0.0 {
                            errors.push(DraftValidationError {
                                field: format!("lines[{}].qty", i),
                                message: "Quantity must be positive".to_string(),
                            });
                        }
                    }
                }
                serde_json::Value::String(s) => {
                    if let Ok(val) = s.parse::<f64>() {
                        if val <= 0.0 {
                            errors.push(DraftValidationError {
                                field: format!("lines[{}].qty", i),
                                message: "Quantity must be positive".to_string(),
                            });
                        }
                    }
                }
                _ => {
                    errors.push(DraftValidationError {
                        field: format!("lines[{}].qty", i),
                        message: "Invalid quantity format".to_string(),
                    });
                }
            }
        }

        match draft.operation_type {
            OperationType::Move => {
                if draft.source_site_id.is_none() {
                    errors.push(DraftValidationError {
                        field: "source_site_id".to_string(),
                        message: "Source site is required for MOVE".to_string(),
                    });
                }
                if draft.destination_site_id.is_none() {
                    errors.push(DraftValidationError {
                        field: "destination_site_id".to_string(),
                        message: "Destination site is required for MOVE".to_string(),
                    });
                }
                if let (Some(src), Some(dst)) = (draft.source_site_id, draft.destination_site_id) {
                    if src == dst {
                        errors.push(DraftValidationError {
                            field: "source_site_id".to_string(),
                            message: "Source and destination sites must differ for MOVE"
                                .to_string(),
                        });
                    }
                }
            }
            OperationType::Issue | OperationType::IssueReturn
                if draft.recipient_id.is_none()
                    && draft.issued_to_name.as_deref().is_none_or(|s| s.is_empty()) =>
            {
                errors.push(DraftValidationError {
                    field: "recipient".to_string(),
                    message: "Recipient or issued_to_name is required for ISSUE/ISSUE_RETURN"
                        .to_string(),
                });
            }
            _ => {}
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
