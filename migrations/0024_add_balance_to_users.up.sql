ALTER TABLE users.tbl_users
ADD COLUMN balance INT8 NOT NULL DEFAULT 0 CHECK (balance >= 0);
