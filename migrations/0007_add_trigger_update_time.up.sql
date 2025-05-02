-- Add up migration script here
-- Trigger để cập nhật updated_at
CREATE TRIGGER update_user_timestamp
    BEFORE UPDATE ON "users"."tbl_users"
    FOR EACH ROW
    EXECUTE FUNCTION "users".update_timestamp();


CREATE TRIGGER update_phone_code_timestamp
    BEFORE UPDATE ON "users"."phone_codes"
    FOR EACH ROW
    EXECUTE FUNCTION "users".update_timestamp();
