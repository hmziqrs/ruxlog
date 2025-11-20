-- Ensure useful extensions exist
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pgcrypto;

DO $$
BEGIN
    IF to_regclass('public.users') IS NULL THEN
        RAISE NOTICE 'Users table not found, skipping seed data.';
        RETURN;
    END IF;

    INSERT INTO users (name, email, password, role)
    SELECT 'Super Admin', 'superadmin@blog.hmziq.rs', 'password123', 'super-admin'
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'superadmin@blog.hmziq.rs');

    INSERT INTO users (name, email, password, role)
    SELECT 'Admin User', 'admin@blog.hmziq.rs', 'password123', 'admin'
    WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'admin@blog.hmziq.rs');
END;
$$;
