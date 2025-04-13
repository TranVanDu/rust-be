-- Add up migration script here
-- Bảng PhoneCodes
CREATE TABLE IF NOT EXISTS "users"."phone_codes"(
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES "users"."tbl_users"(pk_user_id) ON DELETE CASCADE,
    phone VARCHAR(15) NOT NULL,
    code VARCHAR(6) NOT NULL,
    revoked BOOLEAN DEFAULT FALSE,
    last_used_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT valid_expires CHECK (expires_at > created_at)
);
CREATE INDEX idx_phone_codes_user_id ON "users"."phone_codes"(user_id);
