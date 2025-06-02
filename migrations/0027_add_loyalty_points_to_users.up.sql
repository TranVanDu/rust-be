ALTER TABLE users.tbl_users
ADD COLUMN loyalty_points INT8 NOT NULL DEFAULT 0 CHECK (loyalty_points >= 0);

-- Add comment to explain the column
COMMENT ON COLUMN users.tbl_users.loyalty_points IS 'Điểm tích lũy của khách hàng'; 
