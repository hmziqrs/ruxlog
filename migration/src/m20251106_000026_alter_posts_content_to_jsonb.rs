use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Use raw SQL to ensure proper cast with USING clause
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                ALTER TABLE "posts"
                ALTER COLUMN "content" TYPE jsonb USING (
                  CASE
                    WHEN "content" ~ '^[[:space:]]*[\{\[]' THEN "content"::jsonb
                    ELSE '{"time":0,"blocks":[],"version":"2.30.7"}'::jsonb
                  END
                );
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Revert back to text if needed
        manager
            .get_connection()
            .execute_unprepared(
                r#"ALTER TABLE "posts" ALTER COLUMN "content" TYPE text USING "content"::text;"#,
            )
            .await?;
        Ok(())
    }
}
