-- Add down migration script here
ALTER TABLE "users"."services"
DROP COLUMN image,
DROP COLUMN service_type;
