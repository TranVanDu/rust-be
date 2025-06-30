-- Add up migration script here
ALTER TABLE "users"."services"
ADD COLUMN combo_service BOOLEAN DEFAULT FALSE;

ALTER TABLE "users"."service_items"
ADD COLUMN combo_service BOOLEAN DEFAULT FALSE;
