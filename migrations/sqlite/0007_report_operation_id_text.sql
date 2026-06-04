-- Migration 0007: Fix report_item_movement_cache.operation_id type
-- SyncServer now returns UUID strings, not integers

-- SQLite doesn't support ALTER COLUMN, so we recreate the table

ALTER TABLE report_item_movement_cache RENAME TO report_item_movement_cache_old;

CREATE TABLE IF NOT EXISTS report_item_movement_cache (
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

INSERT INTO report_item_movement_cache (
    row_id, params_hash, item_id, item_name, item_sku, unit_symbol,
    operation_type, operation_id, quantity, effective_at, site_id, site_code, refreshed_at
)
SELECT
    row_id, params_hash, item_id, item_name, item_sku, unit_symbol,
    operation_type, CAST(operation_id AS TEXT), quantity, effective_at, site_id, site_code, refreshed_at
FROM report_item_movement_cache_old;

DROP TABLE report_item_movement_cache_old;

CREATE INDEX IF NOT EXISTS idx_im_cache_params ON report_item_movement_cache(params_hash);
