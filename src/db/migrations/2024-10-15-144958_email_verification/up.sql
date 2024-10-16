-- Your SQL goes here
DROP TABLE IF EXISTS "migrations";
ALTER TABLE "users" ADD COLUMN "is_verified" BOOL NOT NULL DEFAULT FALSE;

CREATE TABLE "email_verifications"(
	"id" SERIAL NOT NULL PRIMARY KEY,
	"user_id" INT4 NOT NULL UNIQUE,
	"code" VARCHAR NOT NULL,
	"created_at" TIMESTAMP NOT NULL,
	"updated_at" TIMESTAMP NOT NULL,
	FOREIGN KEY ("user_id") REFERENCES "users"("id"),
	UNIQUE ("user_id", "code")
);
