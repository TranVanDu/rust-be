-- Add migration script here
ALTER TABLE users.tbl_users 
ADD COLUMN full_name VARCHAR(150)
