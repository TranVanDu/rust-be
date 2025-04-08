CREATE TABLE IF NOT EXISTS "users"."refresh_tokens" (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES "users"."tbl_users"(pk_user_id) ON DELETE CASCADE,
    token TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    revoked BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    last_used_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT valid_expires CHECK (expires_at > created_at)
);

CREATE INDEX idx_refresh_tokens_user_id ON "users"."refresh_tokens"(user_id);
CREATE INDEX idx_refresh_tokens_token ON "users"."refresh_tokens"(token);

CREATE OR REPLACE FUNCTION "users".update_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE 'plpgsql';

CREATE TRIGGER update_refresh_tokens_timestamp
    BEFORE UPDATE ON "users"."refresh_tokens"
    FOR EACH ROW
    EXECUTE FUNCTION "users".update_timestamp();
