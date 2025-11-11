use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Creates the `comment_flags` table:
/// - id (pk)
/// - comment_id -> post_comments.id (FK, cascade on delete/update)
/// - user_id -> users.id (FK, cascade on delete/update)
/// - reason (text, nullable)
/// - created_at (timestamptz, default now)
/// Adds a unique constraint on (comment_id, user_id).
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create table
        manager
            .create_table(
                Table::create()
                    .table(CommentFlags::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(CommentFlags::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(CommentFlags::CommentId).integer().not_null())
                    .col(ColumnDef::new(CommentFlags::UserId).integer().not_null())
                    .col(ColumnDef::new(CommentFlags::Reason).text().null())
                    .col(
                        ColumnDef::new(CommentFlags::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_comment_flags_comment")
                            .from(CommentFlags::Table, CommentFlags::CommentId)
                            .to(PostComments::Table, PostComments::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_comment_flags_user")
                            .from(CommentFlags::Table, CommentFlags::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique (comment_id, user_id)
        manager
            .create_index(
                Index::create()
                    .name("ux_comment_flags_comment_user")
                    .table(CommentFlags::Table)
                    .col(CommentFlags::CommentId)
                    .col(CommentFlags::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Helpful lookup index on comment_id
        manager
            .create_index(
                Index::create()
                    .name("idx_comment_flags_comment_id")
                    .table(CommentFlags::Table)
                    .col(CommentFlags::CommentId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop table (indexes/constraints are dropped automatically)
        manager
            .drop_table(Table::drop().table(CommentFlags::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum CommentFlags {
    Table,
    Id,
    CommentId,
    UserId,
    Reason,
    CreatedAt,
}

#[derive(Iden)]
enum PostComments {
    Table,
    Id,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
