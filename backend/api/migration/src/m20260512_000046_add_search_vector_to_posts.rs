use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        // Add search_vector tsvector column
        db.execute(Statement::from_string(
            backend,
            "ALTER TABLE posts ADD COLUMN IF NOT EXISTS search_vector tsvector".to_string(),
        ))
        .await?;

        // Create GIN index for full-text search
        db.execute(Statement::from_string(
            backend,
            "CREATE INDEX IF NOT EXISTS idx_posts_search_vector ON posts USING GIN (search_vector)"
                .to_string(),
        ))
        .await?;

        // Create trigger function to auto-update search_vector
        db.execute(Statement::from_string(
            backend,
            r#"
            CREATE OR REPLACE FUNCTION posts_search_vector_update() RETURNS trigger AS $$
            BEGIN
                NEW.search_vector :=
                    setweight(to_tsvector('english', COALESCE(NEW.title, '')), 'A') ||
                    setweight(to_tsvector('english', COALESCE(NEW.excerpt, '')), 'B') ||
                    setweight(to_tsvector('english', COALESCE(NEW.slug, '')), 'C');
                RETURN NEW;
            END
            $$ LANGUAGE plpgsql
            "#.to_string(),
        ))
        .await?;

        // Create trigger
        db.execute(Statement::from_string(
            backend,
            r#"
            DROP TRIGGER IF EXISTS posts_search_vector_trigger ON posts;
            CREATE TRIGGER posts_search_vector_trigger
                BEFORE INSERT OR UPDATE OF title, excerpt, slug ON posts
                FOR EACH ROW
                EXECUTE FUNCTION posts_search_vector_update()
            "#.to_string(),
        ))
        .await?;

        // Backfill existing posts
        db.execute(Statement::from_string(
            backend,
            r#"
            UPDATE posts SET search_vector =
                setweight(to_tsvector('english', COALESCE(title, '')), 'A') ||
                setweight(to_tsvector('english', COALESCE(excerpt, '')), 'B') ||
                setweight(to_tsvector('english', COALESCE(slug, '')), 'C')
            WHERE search_vector IS NULL
            "#.to_string(),
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        db.execute(Statement::from_string(
            backend,
            "DROP TRIGGER IF EXISTS posts_search_vector_trigger ON posts".to_string(),
        ))
        .await?;

        db.execute(Statement::from_string(
            backend,
            "DROP FUNCTION IF EXISTS posts_search_vector_update()".to_string(),
        ))
        .await?;

        db.execute(Statement::from_string(
            backend,
            "DROP INDEX IF EXISTS idx_posts_search_vector".to_string(),
        ))
        .await?;

        db.execute(Statement::from_string(
            backend,
            "ALTER TABLE posts DROP COLUMN IF EXISTS search_vector".to_string(),
        ))
        .await?;

        Ok(())
    }
}
