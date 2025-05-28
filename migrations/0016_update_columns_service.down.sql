-- Add down migration script here
ALTER TABLE "users"."services"
DROP COLUMN is_signature;

ALTER TABLE "users"."service_items"
DROP COLUMN is_signature;

DROP TABLE IF EXISTS "users"."zalo_tokens";
