ALTER TABLE outbox_events ADD COLUMN command_type TEXT;
ALTER TABLE outbox_events ADD COLUMN idempotency_key TEXT;
ALTER TABLE outbox_events ADD COLUMN payload_hash TEXT;
ALTER TABLE outbox_events ADD COLUMN server_result TEXT;

CREATE INDEX IF NOT EXISTS idx_outbox_command_type ON outbox_events(command_type);
CREATE INDEX IF NOT EXISTS idx_outbox_idempotency ON outbox_events(idempotency_key);
