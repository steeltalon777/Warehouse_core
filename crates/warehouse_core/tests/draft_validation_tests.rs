use warehouse_core::domain::operation::{OperationDraft, OperationDraftLine, OperationType};
use warehouse_core::operations::validation::DraftValidator;

fn make_draft(operation_type: OperationType) -> OperationDraft {
    OperationDraft {
        draft_id: uuid::Uuid::new_v4(),
        operation_type,
        site_id: Some(1),
        lines: vec![OperationDraftLine {
            line_id: uuid::Uuid::new_v4(),
            item_id: Some(1),
            temporary_item: None,
            qty: serde_json::json!("10"),
            batch: None,
            comment: None,
        }],
        effective_at: None,
        source_site_id: None,
        destination_site_id: None,
        recipient_id: None,
        issued_to_name: None,
        comment: None,
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
    }
}

#[test]
fn test_validate_valid_receive() {
    let draft = make_draft(OperationType::Receive);
    assert!(DraftValidator::validate(&draft).is_ok());
}

#[test]
fn test_validate_valid_expense() {
    let mut draft = make_draft(OperationType::Expense);
    draft.site_id = Some(5);
    assert!(DraftValidator::validate(&draft).is_ok());
}

#[test]
fn test_validate_valid_move() {
    let mut draft = make_draft(OperationType::Move);
    draft.source_site_id = Some(1);
    draft.destination_site_id = Some(2);
    assert!(DraftValidator::validate(&draft).is_ok());
}

#[test]
fn test_validate_no_site_id() {
    let mut draft = make_draft(OperationType::Receive);
    draft.site_id = None;
    let err = DraftValidator::validate(&draft).unwrap_err();
    assert!(err.iter().any(|e| e.field == "site_id"));
}

#[test]
fn test_validate_no_lines() {
    let mut draft = make_draft(OperationType::Receive);
    draft.lines.clear();
    let err = DraftValidator::validate(&draft).unwrap_err();
    assert!(err.iter().any(|e| e.field == "lines"));
}

#[test]
fn test_validate_zero_qty() {
    let mut draft = make_draft(OperationType::Receive);
    draft.lines[0].qty = serde_json::json!("0");
    let err = DraftValidator::validate(&draft).unwrap_err();
    assert!(err.iter().any(|e| e.field == "lines[0].qty"));
}

#[test]
fn test_validate_move_same_sites() {
    let mut draft = make_draft(OperationType::Move);
    draft.source_site_id = Some(1);
    draft.destination_site_id = Some(1);
    let err = DraftValidator::validate(&draft).unwrap_err();
    assert!(
        err.iter()
            .any(|e| e.field == "source_site_id" && e.message.contains("must differ"))
    );
}

#[test]
fn test_validate_issue_no_recipient() {
    let draft = make_draft(OperationType::Issue);
    let err = DraftValidator::validate(&draft).unwrap_err();
    assert!(err.iter().any(|e| e.field == "recipient"));
}

#[test]
fn test_validate_issue_with_recipient() {
    let mut draft = make_draft(OperationType::Issue);
    draft.recipient_id = Some(1);
    assert!(DraftValidator::validate(&draft).is_ok());
}

#[test]
fn test_validate_issue_with_issued_to_name() {
    let mut draft = make_draft(OperationType::Issue);
    draft.issued_to_name = Some("Иван Петров".into());
    assert!(DraftValidator::validate(&draft).is_ok());
}
