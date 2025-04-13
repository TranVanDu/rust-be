-- Add down migration script here
ALTER TABLE "users"."refresh_tokens"
DROP COLUMN  device_info
