-- Add migration script here
ALTER TABLE users.refresh_tokens 
ADD COLUMN device_info VARCHAR(150)
