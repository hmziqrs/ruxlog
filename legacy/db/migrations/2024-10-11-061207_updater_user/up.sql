-- Your SQL goes here

-- Make email unique
ALTER TABLE "users"
ADD CONSTRAINT unique_email UNIQUE (email);

-- Add column avatar
ALTER TABLE "users"
ADD COLUMN "avatar" VARCHAR DEFAULT NULL;
