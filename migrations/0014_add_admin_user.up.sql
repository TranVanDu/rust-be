INSERT INTO users.tbl_users (user_name, role, password_hash, is_active, is_verify)
VALUES (
    'admin',
    'ADMIN',
    '$argon2id$v=19$m=19456,t=2,p=1$zsPeAXXDEVtw4/k9WBra9g$ON0/amj0Gceh7oHa0VgGHomWEB163WemGRCLG8mRElw',
    true,
    true
); 
