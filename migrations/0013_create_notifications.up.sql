CREATE TABLE IF NOT EXISTS "users"."notifications"  (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES "users"."tbl_users"(pk_user_id) ON DELETE RESTRICT,
    title VARCHAR(255) NOT NULL,
    body TEXT NOT NULL,
    type VARCHAR(50) NOT NULL,
    data JSONB,
    is_read BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Trigger để cập nhật updated_at
CREATE TRIGGER update_notifications_timestamp 
    BEFORE UPDATE ON "users"."notifications"
    FOR EACH ROW
    EXECUTE FUNCTION "users".update_timestamp();

CREATE INDEX idx_notifications_user_id ON "users"."notifications"(user_id);
