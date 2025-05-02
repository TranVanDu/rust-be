-- Add down migration script here
DROP TRIGGER IF EXISTS update_user_timestamp ON "users"."phone_codes";
DROP TRIGGER IF EXISTS update_phone_code_timestamp ON "users"."tbl_users";
