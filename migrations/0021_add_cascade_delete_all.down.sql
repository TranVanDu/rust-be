-- Add down migration script here
-- For notifications table
ALTER TABLE "users"."notifications" 
DROP CONSTRAINT IF EXISTS "notifications_user_id_fkey",
ADD CONSTRAINT "notifications_user_id_fkey" 
FOREIGN KEY (user_id) 
REFERENCES "users"."tbl_users"(pk_user_id);

-- For notification_tokens table
ALTER TABLE "users"."notification_tokens" 
DROP CONSTRAINT IF EXISTS "notification_tokens_user_id_fkey",
ADD CONSTRAINT "notification_tokens_user_id_fkey" 
FOREIGN KEY (user_id) 
REFERENCES "users"."tbl_users"(pk_user_id);

-- For refresh_tokens table
ALTER TABLE "users"."refresh_tokens" 
DROP CONSTRAINT IF EXISTS "refresh_tokens_user_id_fkey",
ADD CONSTRAINT "refresh_tokens_user_id_fkey" 
FOREIGN KEY (user_id) 
REFERENCES "users"."tbl_users"(pk_user_id);

-- For phone_codes table
ALTER TABLE "users"."phone_codes" 
DROP CONSTRAINT IF EXISTS "phone_codes_user_id_fkey",
ADD CONSTRAINT "phone_codes_user_id_fkey" 
FOREIGN KEY (user_id) 
REFERENCES "users"."tbl_users"(pk_user_id); 
