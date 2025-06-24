-- Drop the SET NULL foreign key constraint
ALTER TABLE "users"."appointments" 
DROP CONSTRAINT IF EXISTS appointments_user_id_fkey;

-- Set user_id back to NOT NULL
ALTER TABLE "users"."appointments"
ALTER COLUMN user_id SET NOT NULL;

-- Add back the original RESTRICT foreign key constraint
ALTER TABLE "users"."appointments"
ADD CONSTRAINT appointments_user_id_fkey 
FOREIGN KEY (user_id) 
REFERENCES "users"."tbl_users"(pk_user_id) 
ON DELETE RESTRICT; 
