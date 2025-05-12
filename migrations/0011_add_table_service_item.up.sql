-- Add up migration script here
-- Tạo bảng tbl_service_items
CREATE TABLE IF NOT EXISTS "users"."service_items" (
    id BIGSERIAL PRIMARY KEY NOT NULL,
    parent_service_id BIGINT NOT NULL REFERENCES "users"."services"(id) ON DELETE CASCADE,
    service_name VARCHAR(100) NOT NULL,
    image TEXT,
    service_name_en TEXT,
    service_name_ko TEXT,
    description TEXT,
    service_type VARCHAR(255),
    description_en TEXT,
    description_ko TEXT,
    price INTEGER NOT NULL CHECK (price >= 0),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Trigger để cập nhật updated_at
CREATE TRIGGER update_service_items_timestamp 
    BEFORE UPDATE ON "users"."service_items"
    FOR EACH ROW
    EXECUTE FUNCTION "users".update_timestamp();

-- Chỉ mục để tối ưu truy vấn
CREATE INDEX idx_service_items_parent_service_id ON "users"."service_items"(parent_service_id);
CREATE INDEX idx_service_items_service_name ON "users"."service_items"(service_name);
