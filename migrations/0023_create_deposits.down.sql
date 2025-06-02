DROP TABLE IF EXISTS "users"."deposits";
DROP TRIGGER IF EXISTS update_deposit_timestamp ON "users"."deposits";
DROP INDEX IF EXISTS idx_deposits_user_id;
DROP INDEX IF EXISTS idx_deposits_status;
DROP INDEX IF EXISTS idx_deposits_created_at;
