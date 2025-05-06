-- Add up migration script here
ALTER TABLE "users"."services"
ADD COLUMN image TEXT,
ADD COLUMN service_type VARCHAR(255);
