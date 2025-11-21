use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::Statement;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Align the media variants table name with the SeaORM model (`media_variants`).
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();
        let sql = r#"
            DO $$
            BEGIN
                IF to_regclass('public.media_variants') IS NULL
                   AND to_regclass('public.media_variant') IS NOT NULL THEN
                    EXECUTE 'ALTER TABLE public.media_variant RENAME TO media_variants';
                END IF;
                IF to_regclass('public.media_variants') IS NULL
                   AND to_regclass('public.media_varaints') IS NOT NULL THEN
                    EXECUTE 'ALTER TABLE public.media_varaints RENAME TO media_variants';
                END IF;
            END$$;
        "#;

        db.execute(Statement::from_string(backend, sql.to_owned()))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let backend = db.get_database_backend();
        let sql = r#"
            DO $$
            BEGIN
                IF to_regclass('public.media_variant') IS NULL
                   AND to_regclass('public.media_variants') IS NOT NULL THEN
                    EXECUTE 'ALTER TABLE public.media_variants RENAME TO media_variant';
                END IF;
            END$$;
        "#;

        db.execute(Statement::from_string(backend, sql.to_owned()))
            .await?;

        Ok(())
    }
}
