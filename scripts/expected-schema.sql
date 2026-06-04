CREATE TABLE auth_context (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE balances (
    site_id INTEGER NOT NULL,
    inventory_subject_id INTEGER NOT NULL,
    qty TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (site_id, inventory_subject_id),
    FOREIGN KEY (inventory_subject_id) REFERENCES inventory_subjects(id)
);
CREATE TABLE categories (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    parent_id INTEGER,
    is_active INTEGER NOT NULL DEFAULT 1,
    path TEXT,
    sort_order INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (parent_id) REFERENCES categories(id)
);
CREATE TABLE conflicts (
    conflict_id TEXT PRIMARY KEY,
    conflict_type TEXT NOT NULL,
    entity_type TEXT,
    entity_id TEXT,
    local_value TEXT,
    server_value TEXT,
    resolution TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    resolved_at TEXT
);
CREATE TABLE documents_cache (
    id TEXT PRIMARY KEY,
    document_type TEXT NOT NULL,
    status TEXT NOT NULL,
    document_number TEXT,
    revision INTEGER NOT NULL DEFAULT 1,
    site_id INTEGER NOT NULL,
    template_name TEXT,
    template_version TEXT,
    payload TEXT,
    created_by_user_id TEXT,
    created_at TEXT NOT NULL,
    finalized_at TEXT,
    updated_at TEXT NOT NULL
);
CREATE TABLE error_log (
    error_id INTEGER PRIMARY KEY AUTOINCREMENT,
    level TEXT NOT NULL DEFAULT 'error',
    category TEXT NOT NULL,
    message TEXT NOT NULL,
    details TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE inventory_subjects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subject_type TEXT NOT NULL CHECK(subject_type IN ('catalog_item','temporary_item')),
    item_id INTEGER,
    temporary_item_id INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE issued_asset_balances (
    operation_line_id TEXT PRIMARY KEY,
    operation_id TEXT NOT NULL,
    site_id INTEGER NOT NULL,
    inventory_subject_id INTEGER NOT NULL,
    qty TEXT NOT NULL,
    issued_to_name TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE TABLE items (
    id INTEGER PRIMARY KEY,
    sku TEXT,
    name TEXT NOT NULL,
    category_id INTEGER NOT NULL,
    unit_id INTEGER NOT NULL,
    description TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    hashtags TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (category_id) REFERENCES categories(id),
    FOREIGN KEY (unit_id) REFERENCES units(id)
);
CREATE TABLE lost_asset_balances (
    operation_line_id TEXT PRIMARY KEY,
    operation_id TEXT NOT NULL,
    site_id INTEGER NOT NULL,
    inventory_subject_id INTEGER NOT NULL,
    lost_qty TEXT NOT NULL,
    is_resolved INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL
);
CREATE TABLE operation_draft_lines (
    line_id TEXT PRIMARY KEY,
    draft_id TEXT NOT NULL,
    item_id INTEGER,
    temp_name TEXT,
    temp_sku TEXT,
    temp_category_id INTEGER,
    temp_unit_id INTEGER,
    temp_description TEXT,
    qty TEXT NOT NULL,
    batch TEXT,
    comment TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (draft_id) REFERENCES operation_drafts(draft_id) ON DELETE CASCADE
);
CREATE TABLE operation_drafts (
    draft_id TEXT PRIMARY KEY,
    operation_type TEXT NOT NULL,
    site_id INTEGER,
    effective_at TEXT,
    source_site_id INTEGER,
    destination_site_id INTEGER,
    recipient_id INTEGER,
    issued_to_name TEXT,
    comment TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE TABLE outbox_events (
    event_uuid TEXT PRIMARY KEY,
    event_type TEXT NOT NULL,
    site_id INTEGER NOT NULL,
    device_id INTEGER,
    payload TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    retry_count INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 5,
    last_error TEXT,
    next_retry_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
, command_type TEXT, idempotency_key TEXT, payload_hash TEXT, server_result TEXT);
CREATE TABLE pending_acceptance_balances (
    operation_id TEXT NOT NULL,
    operation_line_id TEXT NOT NULL,
    site_id INTEGER NOT NULL,
    inventory_subject_id INTEGER NOT NULL,
    qty TEXT NOT NULL,
    accepted_qty TEXT,
    lost_qty TEXT,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (operation_line_id)
);
CREATE TABLE profile_metadata (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
CREATE TABLE recipients (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    recipient_type TEXT NOT NULL,
    contact_info TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE TABLE report_item_movement_cache (
    row_id INTEGER PRIMARY KEY AUTOINCREMENT,
    params_hash TEXT NOT NULL,
    item_id INTEGER NOT NULL,
    item_name TEXT NOT NULL,
    item_sku TEXT,
    unit_symbol TEXT NOT NULL,
    operation_type TEXT NOT NULL,
    operation_id TEXT NOT NULL,
    quantity TEXT NOT NULL,
    effective_at TEXT NOT NULL,
    site_id INTEGER NOT NULL,
    site_code TEXT NOT NULL,
    refreshed_at TEXT NOT NULL
);
CREATE TABLE report_stock_summary_cache (
    row_id INTEGER PRIMARY KEY AUTOINCREMENT,
    params_hash TEXT NOT NULL,
    item_id INTEGER NOT NULL,
    item_name TEXT NOT NULL,
    item_sku TEXT,
    unit_symbol TEXT NOT NULL,
    site_id INTEGER NOT NULL,
    site_code TEXT NOT NULL,
    quantity TEXT NOT NULL,
    last_operation_at TEXT,
    refreshed_at TEXT NOT NULL
);
CREATE TABLE sites (
    site_id INTEGER PRIMARY KEY,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    description TEXT,
    permissions TEXT,
    updated_at TEXT NOT NULL
);
CREATE TABLE sync_cursors (
    cursor_type TEXT PRIMARY KEY,
    cursor_value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE TABLE sync_runs (
    run_id TEXT PRIMARY KEY,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    status TEXT NOT NULL DEFAULT 'running',
    push_count INTEGER NOT NULL DEFAULT 0,
    pull_count INTEGER NOT NULL DEFAULT 0,
    error_count INTEGER NOT NULL DEFAULT 0,
    error TEXT
, families_json TEXT, mode TEXT);
CREATE TABLE temporary_items (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    sku TEXT,
    category_id INTEGER,
    unit_id INTEGER NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    resolved_item_id INTEGER,
    created_by_user_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE TABLE units (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX idx_balances_site ON balances(site_id);
CREATE INDEX idx_categories_parent ON categories(parent_id);
CREATE INDEX idx_categories_updated ON categories(updated_at);
CREATE INDEX idx_conflicts_type ON conflicts(conflict_type);
CREATE INDEX idx_docs_site ON documents_cache(site_id);
CREATE INDEX idx_docs_status ON documents_cache(status);
CREATE INDEX idx_docs_type ON documents_cache(document_type);
CREATE INDEX idx_draft_lines_draft ON operation_draft_lines(draft_id);
CREATE INDEX idx_error_log_created ON error_log(created_at);
CREATE INDEX idx_im_cache_params ON report_item_movement_cache(params_hash);
CREATE INDEX idx_inv_subject_type ON inventory_subjects(subject_type);
CREATE INDEX idx_issued_asset_op ON issued_asset_balances(operation_id);
CREATE INDEX idx_items_category ON items(category_id);
CREATE INDEX idx_items_sku ON items(sku);
CREATE INDEX idx_items_updated ON items(updated_at);
CREATE INDEX idx_lost_asset_op ON lost_asset_balances(operation_id);
CREATE INDEX idx_outbox_command_type ON outbox_events(command_type);
CREATE INDEX idx_outbox_idempotency ON outbox_events(idempotency_key);
CREATE INDEX idx_outbox_retry ON outbox_events(next_retry_at);
CREATE INDEX idx_outbox_status ON outbox_events(status);
CREATE INDEX idx_pending_acc_op ON pending_acceptance_balances(operation_id);
CREATE INDEX idx_ss_cache_params ON report_stock_summary_cache(params_hash);
CREATE INDEX idx_temp_items_status ON temporary_items(status);
CREATE INDEX idx_units_updated ON units(updated_at);
