-- Add membership level column to users table
ALTER TABLE users.tbl_users
ADD COLUMN membership_level VARCHAR(20) NOT NULL DEFAULT 'BRONZE' 
CHECK (membership_level IN ('BRONZE', 'GOLD', 'DIAMOND', 'VIP'));

-- Add comment to explain membership levels
COMMENT ON COLUMN users.tbl_users.membership_level IS 'User membership level: BRONZE (Cơ bản), GOLD (Vàng), DIAMOND (Kim cương), VIP (VIP)'; 
