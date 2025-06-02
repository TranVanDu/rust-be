-- Add deposit_type column to deposits table
ALTER TABLE users.deposits 
ADD COLUMN deposit_type VARCHAR(20) NOT NULL DEFAULT 'DEPOSIT';

-- Add comment to explain the column
COMMENT ON COLUMN users.deposits.deposit_type IS 'Type of deposit: DEPOSIT for adding money, PAYMENT for service payment'; 
