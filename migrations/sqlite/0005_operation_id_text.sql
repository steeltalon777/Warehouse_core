-- Migration 0005: Change asset operation_id/operation_line_id from INTEGER to TEXT
-- SyncServer returns UUID strings for operation IDs (Fix E).

-- pending_acceptance_balances
ALTER TABLE pending_acceptance_balances RENAME TO _pending_old_0005;
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
INSERT INTO pending_acceptance_balances
    (operation_id, operation_line_id, site_id, inventory_subject_id, qty, accepted_qty, lost_qty, updated_at)
    SELECT CAST(operation_id AS TEXT), CAST(operation_line_id AS TEXT), site_id, inventory_subject_id, qty, accepted_qty, lost_qty, updated_at
    FROM _pending_old_0005;
DROP TABLE _pending_old_0005;
CREATE INDEX IF NOT EXISTS idx_pending_acc_op ON pending_acceptance_balances(operation_id);

-- lost_asset_balances
ALTER TABLE lost_asset_balances RENAME TO _lost_old_0005;
CREATE TABLE lost_asset_balances (
    operation_line_id TEXT PRIMARY KEY,
    operation_id TEXT NOT NULL,
    site_id INTEGER NOT NULL,
    inventory_subject_id INTEGER NOT NULL,
    lost_qty TEXT NOT NULL,
    is_resolved INTEGER NOT NULL DEFAULT 0,
    updated_at TEXT NOT NULL
);
INSERT INTO lost_asset_balances
    (operation_line_id, operation_id, site_id, inventory_subject_id, lost_qty, is_resolved, updated_at)
    SELECT CAST(operation_line_id AS TEXT), CAST(operation_id AS TEXT), site_id, inventory_subject_id, lost_qty, is_resolved, updated_at
    FROM _lost_old_0005;
DROP TABLE _lost_old_0005;
CREATE INDEX IF NOT EXISTS idx_lost_asset_op ON lost_asset_balances(operation_id);

-- issued_asset_balances
ALTER TABLE issued_asset_balances RENAME TO _issued_old_0005;
CREATE TABLE issued_asset_balances (
    operation_line_id TEXT PRIMARY KEY,
    operation_id TEXT NOT NULL,
    site_id INTEGER NOT NULL,
    inventory_subject_id INTEGER NOT NULL,
    qty TEXT NOT NULL,
    issued_to_name TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
INSERT INTO issued_asset_balances
    (operation_line_id, operation_id, site_id, inventory_subject_id, qty, issued_to_name, updated_at)
    SELECT CAST(operation_line_id AS TEXT), CAST(operation_id AS TEXT), site_id, inventory_subject_id, qty, issued_to_name, updated_at
    FROM _issued_old_0005;
DROP TABLE _issued_old_0005;
CREATE INDEX IF NOT EXISTS idx_issued_asset_op ON issued_asset_balances(operation_id);
