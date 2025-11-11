-- This file should undo anything in `up.sql`


ALTER TABLE posts DROP CONSTRAINT IF EXISTS unique_tag_ids;
ALTER TABLE posts DROP COLUMN IF EXISTS tag_ids;

-- Then, remove the function
DROP FUNCTION IF EXISTS array_remove_duplicates(anyarray);

DROP TABLE IF EXISTS tags;
