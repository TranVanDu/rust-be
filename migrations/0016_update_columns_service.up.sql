-- Add up migration script here
-- Create zalo_tokens table first
CREATE TABLE IF NOT EXISTS "users"."zalo_tokens" (
    id BIGSERIAL PRIMARY KEY NOT NULL,
    access_token TEXT NOT NULL,
    refresh_token TEXT NOT NULL,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL
);

-- Then insert data
INSERT INTO "users"."zalo_tokens" (access_token, refresh_token, expires_at)
VALUES (
    'vby95U0utcJcNWiNvZBTPUbN3nJmDvWQbIiVQDf7h5-eMpfxxagKVeDCGGdGLv4YlZH51Vilx1_qJZjjf2kzT-itBKoRLxbft4i5PwXmh67T70fHs1wD9RWBMJJT9zb3Z25yGiCqpqQoAHmqm1g_0BORCNRdUkHzkXnPSzqvY5kdLIbQmJAvO9GX2ape1g1tcmOl19yS_WIO37XpuI3NOQPCV1Fo8FO8YJOZ8ye8g2QvQoaIqN2-0fOM5Zdi6y0ibG9B0ziJmGxVTXSksd619DbcLXUy0luDnWTI2BaMrIpVSaKYc1B35S1-P3QJ6_4RtXvnADePhIFV60udkp-49liQCrMJ7OLLy00GPBGvmqVC76D2ht7LIhLdUr7RLgugW6KE1VD_Y2Qd6WqWo7sY1vGG72WtLEH2vINSRm',
    'SaF0BQEEX5DoGi8lfOI7KdrBnYopmCO-QaxbD-dauH8aGCK5c-VHAWWhur62hFqkAXNe0V-opZn8ACyRnuVYDMWWrXVaXz8mG3FVKC-qa5yU0vPUpfU6VqH4cqZAwfXJM732PlFq-1eZGg8yySMo95Lsj1_IwP9XId6mIkp3dq5MPRnPvlU7G6zCicxwdx1bV2RtOD-etsDcBlX-qkIxK4jldMMVzDfxCa2LKh3QftmS5kS9evld3GGSgNZLiwm2ToRZ5kUlf3P04hzCnekEJ4eciJJWluaeTpEBKCBzjoeQQwaJfVsz6dCwZp3ceBCdTM-q6kJcXs5yRfyVwwcZ4Kblx776eFnlHspCGzB5t45vKE5KhPlvJ0KCnpEzfzCsQXFrJEkhwZWcO_yKfU_gA1SYyGU5Z_GTuFvtJQQJXbW',
    NOW() + INTERVAL '1 day'
);

-- Add columns to existing tables
ALTER TABLE "users"."services"
ADD COLUMN is_signature BOOLEAN DEFAULT FALSE;

ALTER TABLE "users"."service_items"
ADD COLUMN is_signature BOOLEAN DEFAULT FALSE;

-- Reset sequences to the max id + 1
SELECT setval('users.services_id_seq', (SELECT MAX(id) FROM users.services));
SELECT setval('users.service_items_id_seq', (SELECT MAX(id) FROM users.service_items));
SELECT setval('users.tbl_users_pk_user_id_seq', (SELECT MAX(pk_user_id) FROM users.tbl_users));


