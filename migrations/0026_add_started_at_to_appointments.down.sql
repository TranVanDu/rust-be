-- Remove started_at column from appointments table
ALTER TABLE users.appointments
DROP COLUMN started_at; 
