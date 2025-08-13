use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        // 1) Create the new enum type for asset context
        db.execute(Statement::from_string(
            backend,
            "CREATE TYPE asset_context AS ENUM (
                'user-avatar',
                'category-cover',
                'category-logo',
                'post-featured',
                'post-inline'
            )"
            .to_owned(),
        ))
        .await?;

        // 2) Normalize existing context values to the new set, accepting legacy -v1 inputs
        db.execute(Statement::from_string(
            backend,
            "
            UPDATE assets
            SET context = CASE
                WHEN context IS NULL THEN NULL
                WHEN lower(context) IN ('user','avatar','profile','user-avatar','user-v1','user-avatar-v1') THEN 'user-avatar'
                WHEN lower(context) IN ('category-cover','cover','cover-image','category','category-v1','category-cover-v1','categories','categories-v1') THEN 'category-cover'
                WHEN lower(context) IN ('category-logo','logo','logo-image','category-logo-v1') THEN 'category-logo'
                WHEN lower(context) IN ('post','featured','featured-image','post-featured','post-v1','post-featured-v1') THEN 'post-featured'
                WHEN lower(context) IN ('inline','inline-image','inline-images','content-image','content-images','post-inline','post-inline-v1') THEN 'post-inline'
                ELSE 'user-avatar'
            END
            WHERE context IS NOT NULL
            "
            .to_owned(),
        ))
        .await?;

        // 3) Alter column type from text to the new enum, casting existing values
        db.execute(Statement::from_string(
            backend,
            "ALTER TABLE assets
             ALTER COLUMN context TYPE asset_context
             USING context::asset_context"
                .to_owned(),
        ))
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();

        // Revert column type back to text
        db.execute(Statement::from_string(
            backend,
            "ALTER TABLE assets
             ALTER COLUMN context TYPE text
             USING context::text"
                .to_owned(),
        ))
        .await?;

        // Drop the enum type
        db.execute(Statement::from_string(
            backend,
            "DROP TYPE IF EXISTS asset_context".to_owned(),
        ))
        .await?;

        Ok(())
    }
}
