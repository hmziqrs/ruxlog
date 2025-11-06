use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop legacy string column if it still exists
        manager
            .get_connection()
            .execute_unprepared(r#"ALTER TABLE posts DROP COLUMN IF EXISTS featured_image;"#)
            .await?;

        // Ensure the new integer column exists
        manager
            .get_connection()
            .execute_unprepared(
                r#"ALTER TABLE posts ADD COLUMN IF NOT EXISTS featured_image_id INTEGER;"#,
            )
            .await?;

        // Add FK constraint unless it has already been created
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                DO $$
                BEGIN
                    IF NOT EXISTS (
                        SELECT 1 FROM information_schema.table_constraints
                        WHERE constraint_name = 'fk_posts_featured_image_media'
                          AND table_name = 'posts'
                    ) THEN
                        ALTER TABLE posts
                        ADD CONSTRAINT fk_posts_featured_image_media
                        FOREIGN KEY (featured_image_id)
                        REFERENCES media(id)
                        ON DELETE SET NULL
                        ON UPDATE CASCADE;
                    END IF;
                END;
                $$;
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop FK if present
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                DO $$
                BEGIN
                    IF EXISTS (
                        SELECT 1 FROM information_schema.table_constraints
                        WHERE constraint_name = 'fk_posts_featured_image_media'
                          AND table_name = 'posts'
                    ) THEN
                        ALTER TABLE posts DROP CONSTRAINT fk_posts_featured_image_media;
                    END IF;
                END;
                $$;
                "#,
            )
            .await?;

        // Drop integer column
        manager
            .get_connection()
            .execute_unprepared(r#"ALTER TABLE posts DROP COLUMN IF EXISTS featured_image_id;"#)
            .await?;

        // Recreate legacy column for rollback compatibility
        manager
            .get_connection()
            .execute_unprepared(
                r#"ALTER TABLE posts ADD COLUMN IF NOT EXISTS featured_image VARCHAR(255);"#,
            )
            .await?;

        Ok(())
    }
}
