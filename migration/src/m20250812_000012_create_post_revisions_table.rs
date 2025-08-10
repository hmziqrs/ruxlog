use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Creates the `post_revisions` table:
/// - id (pk)
/// - post_id -> posts.id (FK, cascade on delete/update)
/// - content (text)
/// - metadata (jsonb, nullable)
/// - created_at (timestamptz)
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create table
        manager
            .create_table(
                Table::create()
                    .table(PostRevisions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PostRevisions::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PostRevisions::PostId).integer().not_null())
                    .col(ColumnDef::new(PostRevisions::Content).text().not_null())
                    .col(ColumnDef::new(PostRevisions::Metadata).json_binary().null())
                    .col(
                        ColumnDef::new(PostRevisions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_revisions_post")
                            .from(PostRevisions::Table, PostRevisions::PostId)
                            .to(Posts::Table, Posts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Helpful index for listing revisions by post and date
        manager
            .create_index(
                Index::create()
                    .name("idx_post_revisions_post_id_created_at")
                    .table(PostRevisions::Table)
                    .col(PostRevisions::PostId)
                    .col(PostRevisions::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop table (indexes/constraints are dropped automatically)
        manager
            .drop_table(Table::drop().table(PostRevisions::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum PostRevisions {
    Table,
    Id,
    PostId,
    Content,
    Metadata,
    CreatedAt,
}

#[derive(Iden)]
enum Posts {
    Table,
    Id,
}
