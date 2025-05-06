-- Add up migration script here
-- Add up migration script here
ALTER TABLE "users"."services"
ADD COLUMN service_name_en TEXT,
ADD COLUMN service_name_ko TEXT,
ADD COLUMN description_ko TEXT,
ADD COLUMN description_en TEXT;
