-- Your SQL goes here


CREATE TABLE "forgot_password"(
	"id" SERIAL NOT NULL PRIMARY KEY,
	"user_id" INT4 NOT NULL UNIQUE,
	"code" VARCHAR NOT NULL,
	"created_at" TIMESTAMP NOT NULL,
	"updated_at" TIMESTAMP NOT NULL,
	FOREIGN KEY ("user_id") REFERENCES "users"("id"),
	UNIQUE ("user_id", "code")
);
