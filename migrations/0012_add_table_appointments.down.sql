-- Add down migration script here
DROP TABLE IF EXISTS "users"."appointments";
DROP TRIGGER IF EXISTS update_appointments_timestamp ON "users"."appointments";
DROP INDEX IF EXISTS idx_appointments_user_id;
DROP INDEX IF EXISTS idx_appointments_status;
DROP INDEX IF EXISTS idx_appointments_start_time;
DROP INDEX IF EXISTS idx_appointments_services_appointment_id;
DROP INDEX IF EXISTS idx_appointments_services_service_id;
DROP INDEX IF EXISTS idx_appointments_services_technician_id;
DROP TABLE IF EXISTS "users"."appointments_services";
DROP TRIGGER IF EXISTS update_appointments_services_timestamp ON "users"."appointments_services";
