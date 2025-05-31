-- Add up migration script here
ALTER TABLE "users"."notifications" 
ALTER COLUMN user_id DROP NOT NULL; 
