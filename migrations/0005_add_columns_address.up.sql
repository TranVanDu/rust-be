-- Add up migration script here
ALTER TABLE "users"."tbl_users"
ADD COLUMN address VARCHAR(200);
