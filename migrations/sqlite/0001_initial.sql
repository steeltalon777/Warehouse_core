-- Migration v1: Initial schema with profile_metadata
CREATE TABLE IF NOT EXISTS profile_metadata (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

INSERT OR IGNORE INTO profile_metadata (key, value) VALUES
    ('schema_version', '1'),
    ('created_at', datetime('now'));
