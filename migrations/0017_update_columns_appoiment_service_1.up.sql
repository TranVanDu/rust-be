-- Add up migration script here
ALTER TABLE "users"."appointments"
ADD COLUMN surcharge INTEGER NOT NULL DEFAULT 0 CHECK (surcharge >= 0),
ADD COLUMN promotion INTEGER NOT NULL DEFAULT 0 CHECK (promotion >= 0),
ADD COLUMN completed_at TIMESTAMP WITH TIME ZONE;

-- Add notification_type column to notifications table
ALTER TABLE "users"."notifications" 
ADD COLUMN notification_type VARCHAR(50) NOT NULL DEFAULT 'APPOINTMENT' 
CHECK (notification_type IN ('APPOINTMENT', 'PROMOTION', 'SURCHARGE', 'PAYMENT', 'SYSTEM'));

-- Rename type column to receiver in notifications table
ALTER TABLE "users"."notifications" 
RENAME COLUMN "type" TO "receiver";

