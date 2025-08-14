use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Alter `post_comments` table to add moderation fields:
/// - hidden (bool, default false)
/// - flags_count (int, default 0)
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add `hidden` and `flags_count` columns
        manager
            .alter_table(
                Table::alter()
                    .table(PostComments::Table)
                    .add_column(
                        ColumnDef::new(PostComments::Hidden)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .add_column(
                        ColumnDef::new(PostComments::FlagsCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop the added columns to revert schema
        manager
            .alter_table(
                Table::alter()
                    .table(PostComments::Table)
                    .drop_column(PostComments::Hidden)
                    .drop_column(PostComments::FlagsCount)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum PostComments {
    Table,
    Id,
    PostId,
    UserId,
    Content,
    LikesCount,
    CreatedAt,
    UpdatedAt,
    Hidden,
    FlagsCount,
}
