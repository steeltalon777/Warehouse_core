-- Migration 0002: Full warehouse client schema
-- Cached domain data, local drafts, outbox, sync state.

-- 1. Auth context
CREATE TABLE IF NOT EXISTS auth_context (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- 2. Sites cache
CREATE TABLE IF NOT EXISTS sites (
    site_id INTEGER PRIMARY KEY,
    code TEXT NOT NULL,
    name TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    description TEXT,
    permissions TEXT,
    updated_at TEXT NOT NULL
);

-- 3. Categories cache
CREATE TABLE IF NOT EXISTS categories (
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
CREATE INDEX IF NOT EXISTS idx_categories_parent ON categories(parent_id);
CREATE INDEX IF NOT EXISTS idx_categories_updated ON categories(updated_at);

-- 4. Units cache
CREATE TABLE IF NOT EXISTS units (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    symbol TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    sort_order INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_units_updated ON units(updated_at);

-- 5. Items cache
CREATE TABLE IF NOT EXISTS items (
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
CREATE INDEX IF NOT EXISTS idx_items_category ON items(category_id);
CREATE INDEX IF NOT EXISTS idx_items_updated ON items(updated_at);
CREATE INDEX IF NOT EXISTS idx_items_sku ON items(sku);

-- 6. Inventory subjects (bridge: catalog_item | temporary_item)
CREATE TABLE IF NOT EXISTS inventory_subjects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subject_type TEXT NOT NULL CHECK(subject_type IN ('catalog_item','temporary_item')),
    item_id INTEGER,
    temporary_item_id INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_inv_subject_type ON inventory_subjects(subject_type);

-- 7. Temporary items cache
CREATE TABLE IF NOT EXISTS temporary_items (
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
CREATE INDEX IF NOT EXISTS idx_temp_items_status ON temporary_items(status);

-- 8. Balances cache
CREATE TABLE IF NOT EXISTS balances (
    site_id INTEGER NOT NULL,
    inventory_subject_id INTEGER NOT NULL,
    qty TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (site_id, inventory_subject_id),
    FOREIGN KEY (inventory_subject_id) REFERENCES inventory_subjects(id)
);
CREATE INDEX IF NOT EXISTS idx_balances_site ON balances(site_id);

-- 9. Pending acceptance cache
CREATE TABLE IF NOT EXISTS pending_acceptance_balances (
    operation_id INTEGER NOT NULL,
    operation_line_id INTEGER NOT NULL,
    site_id INTEGER NOT NULL,
    inventory_subject_id INTEGER NOT NULL,
    qty TEXT NOT NULL,
    accepted_qty TEXT,
    lost_qty TEXT,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (operation_line_id)
);
CREATE INDEX IF NOT EXISTS idx_pending_acc_op ON pending_acceptance_balances(operation_id);

-- 10. Lost asset cache
CREATE TABLE IF NOT EXISTS lost_asset_balances (
    operation_line_id INTEGER PRIMARY KEY,
    operation_id INTEGER NOT NULL,
    site_id INTEGER NOT NULL,
    inventory_subject_id INTEGER NOT NULL,
    lost_qty TEXT NOT NULL,
    is_resolved INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_lost_asset_op ON lost_asset_balances(operation_id);

-- 11. Issued asset cache
CREATE TABLE IF NOT EXISTS issued_asset_balances (
    operation_line_id INTEGER PRIMARY KEY,
    operation_id INTEGER NOT NULL,
    site_id INTEGER NOT NULL,
    inventory_subject_id INTEGER NOT NULL,
    qty TEXT NOT NULL,
    issued_to_name TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_issued_asset_op ON issued_asset_balances(operation_id);

-- 12. Recipients cache
CREATE TABLE IF NOT EXISTS recipients (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    recipient_type TEXT NOT NULL,
    contact_info TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 13. Operation drafts
CREATE TABLE IF NOT EXISTS operation_drafts (
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

-- 14. Operation draft lines
CREATE TABLE IF NOT EXISTS operation_draft_lines (
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
CREATE INDEX IF NOT EXISTS idx_draft_lines_draft ON operation_draft_lines(draft_id);

-- 15. Outbox events (push queue)
CREATE TABLE IF NOT EXISTS outbox_events (
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
);
CREATE INDEX IF NOT EXISTS idx_outbox_status ON outbox_events(status);
CREATE INDEX IF NOT EXISTS idx_outbox_retry ON outbox_events(next_retry_at);

-- 16. Sync cursors (pull tracking)
CREATE TABLE IF NOT EXISTS sync_cursors (
    cursor_type TEXT PRIMARY KEY,
    cursor_value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 17. Sync run history
CREATE TABLE IF NOT EXISTS sync_runs (
    run_id TEXT PRIMARY KEY,
    started_at TEXT NOT NULL,
    finished_at TEXT,
    status TEXT NOT NULL DEFAULT 'running',
    push_count INTEGER NOT NULL DEFAULT 0,
    pull_count INTEGER NOT NULL DEFAULT 0,
    error_count INTEGER NOT NULL DEFAULT 0,
    error TEXT
);

-- 18. Conflict log
CREATE TABLE IF NOT EXISTS conflicts (
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
CREATE INDEX IF NOT EXISTS idx_conflicts_type ON conflicts(conflict_type);

-- 19. Error log
CREATE TABLE IF NOT EXISTS error_log (
    error_id INTEGER PRIMARY KEY AUTOINCREMENT,
    level TEXT NOT NULL DEFAULT 'error',
    category TEXT NOT NULL,
    message TEXT NOT NULL,
    details TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_error_log_created ON error_log(created_at);
