use warehouse_core::domain::assets::{IssuedAssetRow, LostAssetRow, PendingAcceptanceRow};
use warehouse_core::domain::auth::AuthSiteInfo;
use warehouse_core::domain::balance::{BalanceRow, BalanceSummaryRow};
use warehouse_core::domain::catalog::{CategoryDto, ItemDto, UnitDto};
use warehouse_core::domain::issue_objects::{IssueObjectDto, IssueObjectTreeDto};
use warehouse_core::domain::operation::{
    AcceptLinesRequest, OperationCreate, OperationDraft, OperationType,
};
use warehouse_core::domain::recipient::{RecipientDto, RecipientType};
use warehouse_core::domain::reports::StockSummaryRow;

// ── Operation DTOs ────────────────────────────────────────────────

#[test]
fn test_operation_type_serde() {
    let cases = vec![
        (r#""RECEIVE""#, OperationType::Receive),
        (r#""EXPENSE""#, OperationType::Expense),
        (r#""WRITE_OFF""#, OperationType::WriteOff),
        (r#""MOVE""#, OperationType::Move),
        (r#""ADJUSTMENT""#, OperationType::Adjustment),
        (r#""ISSUE""#, OperationType::Issue),
        (r#""ISSUE_RETURN""#, OperationType::IssueReturn),
    ];
    for (json, expected) in &cases {
        let deserialized: OperationType = serde_json::from_str(json).unwrap();
        assert_eq!(&deserialized, expected);
        let serialized = serde_json::to_string(&deserialized).unwrap();
        assert_eq!(&serialized, *json);
    }
}

#[test]
fn test_operation_draft_serde() {
    let draft = OperationDraft {
        draft_id: uuid::Uuid::nil(),
        operation_type: OperationType::Receive,
        site_id: Some(1),
        lines: vec![],
        effective_at: None,
        source_site_id: None,
        destination_site_id: None,
        recipient_id: None,
        issued_to_name: None,
        comment: None,
        created_at: "2024-01-01T00:00:00Z".to_string(),
        updated_at: "2024-01-01T00:00:00Z".to_string(),
    };
    let json = serde_json::to_string(&draft).unwrap();
    let deserialized: OperationDraft = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.operation_type, OperationType::Receive);
    assert_eq!(deserialized.site_id, Some(1));
    assert_eq!(deserialized.draft_id, uuid::Uuid::nil());
}

#[test]
fn test_operation_create_serde() {
    let create = OperationCreate {
        operation_type: OperationType::Expense,
        site_id: 2,
        lines: vec![warehouse_core::domain::operation::OperationLineCreate {
            item_id: 10,
            qty: serde_json::json!("3.5"),
            batch: None,
            comment: Some("test".into()),
        }],
        effective_at: Some("2024-06-01T00:00:00Z".to_string()),
        source_site_id: None,
        destination_site_id: None,
        recipient_id: None,
        issued_to_name: None,
        comment: None,
    };
    let json = serde_json::to_string(&create).unwrap();
    let deserialized: OperationCreate = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.operation_type, OperationType::Expense);
    assert_eq!(deserialized.lines.len(), 1);
    assert_eq!(deserialized.lines[0].item_id, 10);
}

#[test]
fn test_accept_lines_request_serde() {
    let req = AcceptLinesRequest {
        lines: vec![warehouse_core::domain::operation::AcceptLineRequest {
            line_id: "line-1".into(),
            accepted_qty: serde_json::json!("5"),
            lost_qty: Some(serde_json::json!("0")),
        }],
    };
    let json = serde_json::to_string(&req).unwrap();
    let deserialized: AcceptLinesRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.lines.len(), 1);
    assert_eq!(deserialized.lines[0].line_id, "line-1");
}

// ── Catalog DTOs ──────────────────────────────────────────────────

#[test]
fn test_item_dto_serde() {
    let item = ItemDto {
        id: 1,
        sku: Some("SKU-001".into()),
        name: "Test Item".into(),
        category_id: 5,
        unit_id: 3,
        description: Some("A test".into()),
        is_active: true,
        hashtags: Some(vec!["tag1".into(), "tag2".into()]),
        updated_at: "2024-01-01T00:00:00Z".into(),
        created_at: Some("2024-01-01T00:00:00Z".into()),
        created_by_user_id: Some("user-1".into()),
        updated_by_user_id: None,
        created_by_user_name: Some("admin".into()),
        updated_by_user_name: None,
    };
    let json = serde_json::to_string(&item).unwrap();
    let deserialized: ItemDto = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, 1);
    assert_eq!(deserialized.name, "Test Item");
    assert_eq!(deserialized.sku, Some("SKU-001".into()));
}

#[test]
fn test_category_dto_serde() {
    let cat = CategoryDto {
        id: 2,
        name: "Electronics".into(),
        parent_id: Some(1),
        is_active: true,
        updated_at: "2024-01-01T00:00:00Z".into(),
        created_by_user_id: None,
        updated_by_user_id: None,
        created_by_user_name: None,
        updated_by_user_name: None,
    };
    let json = serde_json::to_string(&cat).unwrap();
    let deserialized: CategoryDto = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "Electronics");
    assert_eq!(deserialized.parent_id, Some(1));
}

#[test]
fn test_unit_dto_serde() {
    let unit = UnitDto {
        id: 3,
        name: "Kilogram".into(),
        symbol: "kg".into(),
        is_active: true,
        updated_at: "2024-01-01T00:00:00Z".into(),
        created_by_user_id: None,
        updated_by_user_id: None,
        created_by_user_name: None,
        updated_by_user_name: None,
    };
    let json = serde_json::to_string(&unit).unwrap();
    let deserialized: UnitDto = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "Kilogram");
    assert_eq!(deserialized.symbol, "kg");
}

// ── Balance DTOs ──────────────────────────────────────────────────

#[test]
fn test_balance_row_serde() {
    let row = BalanceRow {
        site_id: 1,
        site_code: "WH-1".into(),
        inventory_subject_id: 100,
        subject_type: "catalog_item".into(),
        item_id: 42,
        temporary_item_id: None,
        item_name: "Widget".into(),
        item_sku: Some("WGT-001".into()),
        unit_symbol: "pcs".into(),
        qty: serde_json::json!("15.0"),
        updated_at: "2024-01-01T00:00:00Z".into(),
        is_subject: Some(true),
    };
    let json = serde_json::to_string(&row).unwrap();
    let deserialized: BalanceRow = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.site_id, 1);
    assert_eq!(deserialized.item_id, 42);
    assert_eq!(deserialized.qty, serde_json::json!("15.0"));
}

#[test]
fn test_balance_summary_serde() {
    let summary = BalanceSummaryRow {
        accessible_sites_count: 3,
        summary: warehouse_core::domain::balance::BalanceSummaryData {
            rows_count: 100,
            sites_count: 3,
            total_quantity: 450.5,
        },
    };
    let json = serde_json::to_string(&summary).unwrap();
    let deserialized: BalanceSummaryRow = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.accessible_sites_count, 3);
    assert_eq!(deserialized.summary.rows_count, 100);
}

// ── Asset DTOs ────────────────────────────────────────────────────

#[test]
fn test_pending_acceptance_serde() {
    let minimal = r#"{"operation_id":"1","operation_line_id":"2","item_id":0,"qty":"10"}"#;
    let row: PendingAcceptanceRow = serde_json::from_str(minimal).unwrap();
    assert_eq!(row.operation_id, "1");
    assert_eq!(row.operation_line_id, "2");
    assert_eq!(row.item_name, "");
    assert!(row.accepted_qty.is_none());

    let json = serde_json::to_string(&row).unwrap();
    let deserialized: PendingAcceptanceRow = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.operation_id, "1");
}

#[test]
fn test_lost_asset_serde() {
    let row = LostAssetRow {
        operation_id: "op-1".into(),
        operation_line_id: "line-1".into(),
        item_id: 1,
        item_name: "Lost Hammer".into(),
        item_sku: Some("HAM-001".into()),
        unit_symbol: "pcs".into(),
        qty: serde_json::json!("2"),
        lost_qty: serde_json::json!("2"),
        is_resolved: false,
    };
    let json = serde_json::to_string(&row).unwrap();
    let deserialized: LostAssetRow = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.operation_id, "op-1");
    assert_eq!(deserialized.item_name, "Lost Hammer");
}

#[test]
fn test_issued_asset_serde() {
    let row = IssuedAssetRow {
        operation_id: "op-2".into(),
        operation_line_id: "line-2".into(),
        item_id: 1,
        item_name: "Issued Tool".into(),
        item_sku: None,
        unit_symbol: "pcs".into(),
        qty: serde_json::json!("1"),
        issued_to_name: "Ivan".into(),
    };
    let json = serde_json::to_string(&row).unwrap();
    let deserialized: IssuedAssetRow = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.issued_to_name, "Ivan");
}

// ── Recipient DTOs ────────────────────────────────────────────────

#[test]
fn test_recipient_serde() {
    let r = RecipientDto {
        id: 1,
        name: "ООО Ромашка".into(),
        recipient_type: RecipientType::Contractor,
        contact_info: Some("roma@example.com".into()),
        is_active: true,
        created_at: "2024-01-01T00:00:00Z".into(),
        updated_at: "2024-01-01T00:00:00Z".into(),
    };
    let json = serde_json::to_string(&r).unwrap();
    let deserialized: RecipientDto = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "ООО Ромашка");
    assert_eq!(deserialized.recipient_type, RecipientType::Contractor);
}

// ── Issue Object DTOs ─────────────────────────────────────────────

#[test]
fn test_issue_object_serde() {
    let obj = IssueObjectDto {
        id: 1,
        display_name: "Станок фрезерный".into(),
        object_type: "machine".into(),
        code: Some("MC-001".into()),
        normalized_key: "stanok-frezernyj".into(),
        comment: Some("Основной цех".into()),
        category_id: Some(10),
        is_active: true,
        merged_into_id: None,
        created_at: "2024-01-01T00:00:00Z".into(),
        updated_at: "2024-01-01T00:00:00Z".into(),
        deleted_at: None,
        deleted_by_user_id: None,
    };
    let json = serde_json::to_string(&obj).unwrap();
    let deserialized: IssueObjectDto = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.display_name, "Станок фрезерный");
    assert_eq!(deserialized.object_type, "machine");
}

#[test]
fn test_issue_object_tree_serde() {
    let tree = IssueObjectTreeDto {
        id: 1,
        r#type: "category".into(),
        name: "Tools".into(),
        comment: None,
        category_id: None,
        children: vec![IssueObjectTreeDto {
            id: 2,
            r#type: "object".into(),
            name: "Hammer".into(),
            comment: Some("Big".into()),
            category_id: Some(1),
            children: vec![],
        }],
    };
    let json = serde_json::to_string(&tree).unwrap();
    let deserialized: IssueObjectTreeDto = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name, "Tools");
    assert_eq!(deserialized.children.len(), 1);
    assert_eq!(deserialized.children[0].name, "Hammer");
}

// ── Report DTOs ───────────────────────────────────────────────────

#[test]
fn test_stock_summary_serde() {
    let row = StockSummaryRow {
        item_id: 1,
        item_name: "Widget".into(),
        item_sku: Some("WGT-001".into()),
        unit_symbol: "pcs".into(),
        site_id: 1,
        site_code: "WH-1".into(),
        quantity: serde_json::json!("100"),
        last_operation_at: Some("2024-01-01T00:00:00Z".into()),
    };
    let json = serde_json::to_string(&row).unwrap();
    let deserialized: StockSummaryRow = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.item_name, "Widget");
    assert_eq!(deserialized.quantity, serde_json::json!("100"));
}

// ── Auth DTOs ─────────────────────────────────────────────────────

#[test]
fn test_auth_site_info_serde() {
    let mut permissions = std::collections::HashMap::new();
    permissions.insert("catalog_read".into(), true);
    permissions.insert("operations_write".into(), false);
    let info = AuthSiteInfo {
        site_id: 1,
        code: "WH-1".into(),
        name: "Main Warehouse".into(),
        permissions,
    };
    let json = serde_json::to_string(&info).unwrap();
    let deserialized: AuthSiteInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.site_id, 1);
    assert_eq!(deserialized.code, "WH-1");
    assert_eq!(deserialized.permissions.get("catalog_read"), Some(&true));
}
