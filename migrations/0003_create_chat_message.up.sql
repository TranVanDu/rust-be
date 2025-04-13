-- Add up migration script here
CREATE TABLE IF NOT EXISTS "users"."chat_messages" (
    id BIGSERIAL PRIMARY KEY,
    sender_id BIGINT NOT NULL REFERENCES "users"."tbl_users"(pk_user_id) ON DELETE CASCADE,
    receiver_id BIGINT NOT NULL REFERENCES "users"."tbl_users"(pk_user_id) ON DELETE CASCADE,
    message TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_chat_messages_sender_id ON "users"."chat_messages"(sender_id);
CREATE INDEX idx_chat_messages_receiver_id ON "users"."chat_messages"(receiver_id);
