CREATE TABLE IF NOT EXISTS "users"."notifications"  (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES "users"."tbl_users"(pk_user_id) ON DELETE RESTRICT,
    appointment_id BIGINT REFERENCES "users"."appointments"(id) ON DELETE SET NULL,
    title VARCHAR(255) NOT NULL,
    body TEXT NOT NULL,
    type VARCHAR(50) NOT NULL,
    data JSONB,
    is_read BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS "users"."notification_tokens"  (
    id BIGSERIAL PRIMARY KEY,
    platform VARCHAR(50) NOT NULL,
    user_id BIGINT NOT NULL REFERENCES "users"."tbl_users"(pk_user_id) ON DELETE RESTRICT,
    token VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

ALTER TABLE "users"."appointments" DROP CONSTRAINT IF EXISTS appointments_status_check;
ALTER TABLE "users"."appointments" ADD CONSTRAINT appointments_status_check 
    CHECK (status IN ('PENDING', 'CONFIRMED', 'IN_PROGRESS', 'COMPLETED', 'CANCELLED', 'PAYMENT'));

-- Trigger để cập nhật updated_at
CREATE TRIGGER update_notifications_timestamp 
    BEFORE UPDATE ON "users"."notifications"
    FOR EACH ROW
    EXECUTE FUNCTION "users".update_timestamp();

CREATE TRIGGER update_notification_tokens_timestamp 
    BEFORE UPDATE ON "users"."notification_tokens"
    FOR EACH ROW
    EXECUTE FUNCTION "users".update_timestamp();

CREATE INDEX idx_notifications_user_id ON "users"."notifications"(user_id);
CREATE INDEX idx_notification_tokens_user_id ON "users"."notification_tokens"(user_id);
