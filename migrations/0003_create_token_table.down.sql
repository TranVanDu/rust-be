-- Add down migration script here
DROP TRIGGER IF EXISTS update_refresh_tokens_timestamp ON "users"."refresh_tokens";
DROP FUNCTION IF EXISTS "users".update_timestamp();
DROP TABLE IF EXISTS "users"."refresh_tokens";
