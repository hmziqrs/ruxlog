-- This file should undo anything in `up.sql`


-- Drop the indexes
DROP INDEX IF EXISTS idx_posts_published_at;
DROP INDEX IF EXISTS idx_posts_created_at;
DROP INDEX IF EXISTS idx_posts_category_id;
DROP INDEX IF EXISTS idx_posts_author_id;

-- Drop the posts table
DROP TABLE IF EXISTS posts;
```
