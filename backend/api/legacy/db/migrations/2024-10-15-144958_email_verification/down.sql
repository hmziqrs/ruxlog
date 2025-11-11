-- This file should undo anything in `up.sql`
CREATE TABLE "migrations"(
	"name" TEXT NOT NULL PRIMARY KEY,
	"executed_at" TIMESTAMP NOT NULL
);

ALTER TABLE "users" DROP COLUMN "is_verified";

DROP TABLE IF EXISTS "email_verifications";
