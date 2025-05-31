-- Add up migration script here
ALTER TABLE "users"."appointments"
ADD COLUMN price INTEGER NOT NULL DEFAULT 0 CHECK (price >= 0); 
