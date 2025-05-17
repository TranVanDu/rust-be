-- Add down migration script here
ALTER TABLE "users"."services"
DROP COLUMN service_name_en,
DROP COLUMN service_name_ko,
DROP COLUMN description_en,
DROP COLUMN description_ko,
DROP COLUMN has_child;
