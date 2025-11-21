-- Ensure useful extensions exist
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pgcrypto;

DO $$
BEGIN
    IF to_regclass('public.users') IS NULL THEN
        RAISE NOTICE 'Users table not found, skipping seed data.';
        RETURN;
    END IF;

    INSERT INTO users (name, email, password, role, is_verified)
    SELECT 'Super Admin', 'superadmin@blog.hmziq.rs', '$argon2id$v=19$m=19456,t=2,p=1$lSCKYb1/M4+3K6ISk0yThw$qHCi4SuMio5DkVgZFHOjVglYv/PiIOdm08fQIMMP5w4', 'super-admin', true
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'superadmin@blog.hmziq.rs');

    INSERT INTO users (name, email, password, role, is_verified)
    SELECT 'Admin User', 'admin@blog.hmziq.rs', '$argon2id$v=19$m=19456,t=2,p=1$cUZvIEd5Elez2IPBLALqiw$lZ40B9v+Vg5zm0eyY4VkqPGBXOi6l1/9n+dN0Fv/hug', 'admin', true
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'admin@blog.hmziq.rs');
END;
$$;
