-- Your SQL goes here

CREATE TYPE user_role AS ENUM ('super-admin', 'admin', 'moderator', 'author', 'user');

-- Add the column with the new enum type
ALTER TABLE "users" ADD COLUMN "role" user_role NOT NULL DEFAULT 'user';
