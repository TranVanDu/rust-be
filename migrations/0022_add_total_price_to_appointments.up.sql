-- Add up migration script here

ALTER TABLE "users"."appointments"
ADD COLUMN total_price INT8 NOT NULL DEFAULT 0 CHECK (total_price >= 0);
