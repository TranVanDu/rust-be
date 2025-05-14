-- Add up migration script here
CREATE TABLE IF NOT EXISTS "users"."appointments" (
    id BIGSERIAL PRIMARY KEY NOT NULL,
    user_id BIGINT NOT NULL REFERENCES "users"."tbl_users"(pk_user_id) ON DELETE RESTRICT,
    receptionist_id BIGINT REFERENCES "users"."tbl_users"(pk_user_id) ON DELETE SET NULL,
    technician_id BIGINT REFERENCES "users"."tbl_users"(pk_user_id) ON DELETE SET NULL,
    start_time VARCHAR(30) NOT NULL,
    end_time VARCHAR(30),
    updated_by BIGINT REFERENCES "users"."tbl_users"(pk_user_id) ON DELETE SET NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'PENDING' CHECK (status IN ('PENDING', 'CONFIRMED', 'IN_PROGRESS', 'COMPLETED', 'CANCELLED')),
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Tạo bảng phụ appointments_services
CREATE TABLE IF NOT EXISTS "users"."appointments_services" (
    id BIGSERIAL PRIMARY KEY NOT NULL,
    appointment_id BIGINT NOT NULL REFERENCES "users"."appointments"(id) ON DELETE CASCADE,
    service_id BIGINT NOT NULL REFERENCES "users"."service_items"(id) ON DELETE RESTRICT,
    technician_id BIGINT REFERENCES "users"."tbl_users"(pk_user_id) ON DELETE SET NULL,
    quantity INT DEFAULT 1,
    sequence INT DEFAULT 1,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_by BIGINT REFERENCES "users"."tbl_users"(pk_user_id) ON DELETE SET NULL
);


-- Trigger để cập nhật updated_at
CREATE TRIGGER update_appointments_timestamp 
    BEFORE UPDATE ON "users"."appointments"
    FOR EACH ROW
    EXECUTE FUNCTION "users".update_timestamp();

CREATE TRIGGER update_appointments_services_timestamp 
    BEFORE UPDATE ON "users"."appointments_services"
    FOR EACH ROW
    EXECUTE FUNCTION "users".update_timestamp();


CREATE INDEX IF NOT EXISTS idx_appointments_user_id ON "users"."appointments" (user_id);
CREATE INDEX IF NOT EXISTS idx_appointments_status ON "users"."appointments" (status);
CREATE INDEX IF NOT EXISTS idx_appointments_start_time ON "users"."appointments" (start_time);

-- Chỉ mục cho appointments_services
CREATE INDEX IF NOT EXISTS idx_appointments_services_appointment_id ON "users"."appointments_services" (appointment_id);
CREATE INDEX IF NOT EXISTS idx_appointments_services_service_id ON "users"."appointments_services" (service_id);
CREATE INDEX IF NOT EXISTS idx_appointments_services_technician_id ON "users"."appointments_services" (technician_id);
