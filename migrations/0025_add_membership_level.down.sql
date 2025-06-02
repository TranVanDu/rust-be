-- Remove membership level column from users table
ALTER TABLE users.tbl_users
DROP COLUMN membership_level; 
