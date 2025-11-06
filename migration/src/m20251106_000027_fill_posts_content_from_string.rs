use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Promote any jsonb string content to a minimal Editor.js structure,
        // embedding the original string as the first paragraph's text.
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                UPDATE "posts"
                SET "content" = jsonb_build_object(
                    'time', (extract(epoch from now()) * 1000)::bigint,
                    'version', '2.30.7',
                    'blocks', jsonb_build_array(
                        jsonb_build_object(
                            'type', 'paragraph',
                            'data', jsonb_build_object(
                                'text', to_jsonb(trim(both '"' from content::text))
                            )
                        )
                    )
                )
                WHERE jsonb_typeof("content") = 'string';
                "#,
            )
            .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        // No-op: keep structured content.
        Ok(())
    }
}

