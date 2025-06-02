-- Add down migration script here
ALTER TABLE "users"."notifications" 
DROP CONSTRAINT IF EXISTS notifications_notification_type_check;

ALTER TABLE "users"."notifications" 
ADD CONSTRAINT notifications_notification_type_check 
CHECK (notification_type IN ('APPOINTMENT', 'PROMOTION', 'SURCHARGE', 'PAYMENT', 'SYSTEM')); 
