-- Insert super-admin user if not exists
INSERT INTO users (name, email, password, role)
SELECT 'Super Admin', 'superadmin@blog.hmziq.rs', 'password123', 'super-admin'
WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'superadmin@blog.hmziq.rs');

-- Insert admin user if not exists
INSERT INTO users (name, email, password, role)
SELECT 'Admin User', 'admin@blog.hmziq.rs', 'password123', 'admin'
WHERE NOT EXISTS (SELECT 1 FROM users WHERE email = 'admin@blog.hmziq.rs');

-- SELECT * from users;/