-- Add down migration script here
ALTER TABLE "users"."appointments"
DROP COLUMN price; 
