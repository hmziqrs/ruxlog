-- This file should undo anything in `up.sql`

DROP TABLE tags;

ALTER TABLE posts
DROP COLUMN tag_ids;
