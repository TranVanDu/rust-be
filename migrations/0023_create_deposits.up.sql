-- Create deposits table
CREATE TABLE IF NOT EXISTS "users"."deposits" (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL REFERENCES users.tbl_users(pk_user_id),
    amount BIGINT NOT NULL,
    status TEXT NOT NULL DEFAULT 'PENDING' CHECK (status IN ('PENDING', 'COMPLETED', 'CANCELLED')),
    payment_method VARCHAR(50) NOT NULL,
    transaction_id VARCHAR(100),
    notes TEXT,
    created_by BIGINT NOT NULL REFERENCES users.tbl_users(pk_user_id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Add indexes
CREATE INDEX idx_deposits_user_id ON users.deposits(user_id);
CREATE INDEX idx_deposits_status ON users.deposits(status);
CREATE INDEX idx_deposits_created_at ON users.deposits(created_at);


-- Trigger để cập nhật updated_at
CREATE TRIGGER update_deposit_timestamp 
    BEFORE UPDATE ON "users"."deposits"
    FOR EACH ROW
    EXECUTE FUNCTION "users".update_timestamp();
