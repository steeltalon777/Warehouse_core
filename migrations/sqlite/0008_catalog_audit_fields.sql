-- Migration 0008: Add catalog audit user fields (created_by_user_id, updated_by_user_id, created_by_user_name, updated_by_user_name)

ALTER TABLE items ADD COLUMN created_by_user_id TEXT;
ALTER TABLE items ADD COLUMN updated_by_user_id TEXT;
ALTER TABLE items ADD COLUMN created_by_user_name TEXT;
ALTER TABLE items ADD COLUMN updated_by_user_name TEXT;

ALTER TABLE categories ADD COLUMN created_by_user_id TEXT;
ALTER TABLE categories ADD COLUMN updated_by_user_id TEXT;
ALTER TABLE categories ADD COLUMN created_by_user_name TEXT;
ALTER TABLE categories ADD COLUMN updated_by_user_name TEXT;

ALTER TABLE units ADD COLUMN created_by_user_id TEXT;
ALTER TABLE units ADD COLUMN updated_by_user_id TEXT;
ALTER TABLE units ADD COLUMN created_by_user_name TEXT;
ALTER TABLE units ADD COLUMN updated_by_user_name TEXT;
