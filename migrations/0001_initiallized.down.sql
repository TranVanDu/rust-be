DROP TABLE IF EXISTS "users"."tbl_users";
DROP SCHEMA IF EXISTS "users";
DROP TRIGGER IF EXISTS update_refresh_tokens_timestamp ON "users"."refresh_tokens";
DROP FUNCTION IF EXISTS "users".update_timestamp();
DROP TABLE IF EXISTS "users"."refresh_tokens";
