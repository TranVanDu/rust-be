-- Add up migration script here
CREATE TABLE IF NOT EXISTS "users"."services" (
    id BIGSERIAL PRIMARY KEY NOT NULL,
    service_name VARCHAR(100) NOT NULL,
    description TEXT,
    price DECIMAL(10, 2) CHECK (price >= 0 OR price IS NULL),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Trigger để cập nhật updated_at
CREATE TRIGGER update_services_timestamp
    BEFORE UPDATE ON "users"."services"
    FOR EACH ROW
    EXECUTE FUNCTION "users".update_timestamp();

-- Chỉ mục để tìm kiếm nhanh theo service_name
CREATE INDEX idx_services_service_name ON "users"."services"(service_name);
