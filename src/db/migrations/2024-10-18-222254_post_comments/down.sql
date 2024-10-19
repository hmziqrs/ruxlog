-- This file should undo anything in `up.sql`



-- Drop foreign key constraints
ALTER TABLE post_comments
    DROP CONSTRAINT IF EXISTS fk_post_comments_post;

ALTER TABLE post_comments
    DROP CONSTRAINT IF EXISTS fk_post_comments_user;

-- Drop indexes
DROP INDEX IF EXISTS idx_post_comments_post_id;
DROP INDEX IF EXISTS idx_post_comments_user_id;
DROP INDEX IF EXISTS idx_post_comments_created_at;

-- Drop the table
DROP TABLE IF EXISTS post_comments;
