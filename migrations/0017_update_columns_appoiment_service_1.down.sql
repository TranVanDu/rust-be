-- Add down migration script here
ALTER TABLE "users"."appointments"
DROP COLUMN surcharge,
DROP COLUMN promotion,
DROP COLUMN completed_at;

-- Remove notification_type column
ALTER TABLE "users"."notifications" DROP COLUMN notification_type;

-- Remove new notification type constraint
ALTER TABLE "users"."notifications" DROP CONSTRAINT IF EXISTS notifications_type_check;

-- Rename receiver column back to type in notifications table
ALTER TABLE "users"."notifications" 
RENAME COLUMN "receiver" TO "type";
