-- Drop existing foreign key constraint
ALTER TABLE "users"."appointments" 
DROP CONSTRAINT IF EXISTS appointments_user_id_fkey;

-- Allow user_id to be NULL
ALTER TABLE "users"."appointments"
ALTER COLUMN user_id DROP NOT NULL;

-- Add new foreign key constraint with ON DELETE SET NULL
ALTER TABLE "users"."appointments"
ADD CONSTRAINT appointments_user_id_fkey 
FOREIGN KEY (user_id) 
REFERENCES "users"."tbl_users"(pk_user_id) 
ON DELETE SET NULL; 
