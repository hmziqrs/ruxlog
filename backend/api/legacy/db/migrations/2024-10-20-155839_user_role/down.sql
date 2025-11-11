-- This file should undo anything in `up.sql`

ALTER TABLE "users" DROP COLUMN "role";

DROP TYPE user_role;
