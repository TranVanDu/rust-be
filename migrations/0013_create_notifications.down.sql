DROP TABLE IF EXISTS notifications; 
DROP TRIGGER IF EXISTS update_notifications_timestamp ON notifications;
DROP INDEX IF EXISTS idx_notifications_user_id;
