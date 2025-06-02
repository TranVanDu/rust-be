-- Add WITHDRAW type to deposit_type check constraint
ALTER TABLE users.deposits
DROP CONSTRAINT IF EXISTS check_deposit_type;

ALTER TABLE users.deposits
ADD CONSTRAINT check_deposit_type 
CHECK (deposit_type IN ('DEPOSIT', 'PAYMENT', 'WITHDRAW'));

-- Update comment to include WITHDRAW type
COMMENT ON COLUMN users.deposits.deposit_type IS 'Type of deposit: DEPOSIT for adding money, PAYMENT for service payment, WITHDRAW for withdrawing money'; 
