DROP TABLE IF EXISTS notifications; 
DROP TRIGGER IF EXISTS update_notifications_timestamp ON notifications;
DROP INDEX IF EXISTS idx_notifications_user_id;
DROP TABLE IF EXISTS notification_tokens;
DROP TRIGGER IF EXISTS update_notification_tokens_timestamp ON notification_tokens;
DROP INDEX IF EXISTS idx_notification_tokens_user_id;
