-- Migration 0003: Documents cache and report cache tables

CREATE TABLE IF NOT EXISTS documents_cache (
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
CREATE INDEX IF NOT EXISTS idx_docs_site ON documents_cache(site_id);
CREATE INDEX IF NOT EXISTS idx_docs_type ON documents_cache(document_type);
CREATE INDEX IF NOT EXISTS idx_docs_status ON documents_cache(status);

CREATE TABLE IF NOT EXISTS report_item_movement_cache (
    row_id INTEGER PRIMARY KEY AUTOINCREMENT,
    params_hash TEXT NOT NULL,
    item_id INTEGER NOT NULL,
    item_name TEXT NOT NULL,
    item_sku TEXT,
    unit_symbol TEXT NOT NULL,
    operation_type TEXT NOT NULL,
    operation_id INTEGER NOT NULL,
    quantity TEXT NOT NULL,
    effective_at TEXT NOT NULL,
    site_id INTEGER NOT NULL,
    site_code TEXT NOT NULL,
    refreshed_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_im_cache_params ON report_item_movement_cache(params_hash);

CREATE TABLE IF NOT EXISTS report_stock_summary_cache (
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
CREATE INDEX IF NOT EXISTS idx_ss_cache_params ON report_stock_summary_cache(params_hash);
