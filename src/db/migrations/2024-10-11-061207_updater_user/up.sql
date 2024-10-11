-- Your SQL goes here
-- make email unique
-- add column avatar
ALTER TABLE "users"
ADD CONSTRAINT unique_email UNIQUE (email)
ADD COLUMN "avatar" VARCHAR DEFAULT NULL;
