use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PostLikes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PostLikes::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PostLikes::PostId).integer().not_null())
                    .col(ColumnDef::new(PostLikes::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(PostLikes::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_likes_post")
                            .from(PostLikes::Table, PostLikes::PostId)
                            .to(Posts::Table, Posts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_likes_user")
                            .from(PostLikes::Table, PostLikes::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique constraint to prevent duplicate likes
        manager
            .create_index(
                Index::create()
                    .name("idx_post_likes_user_post_unique")
                    .table(PostLikes::Table)
                    .col(PostLikes::PostId)
                    .col(PostLikes::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Create index for faster lookups by post
        manager
            .create_index(
                Index::create()
                    .name("idx_post_likes_post_id")
                    .table(PostLikes::Table)
                    .col(PostLikes::PostId)
                    .to_owned(),
            )
            .await?;

        // Create index for faster lookups by user
        manager
            .create_index(
                Index::create()
                    .name("idx_post_likes_user_id")
                    .table(PostLikes::Table)
                    .col(PostLikes::UserId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PostLikes::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum PostLikes {
    Table,
    Id,
    PostId,
    UserId,
    CreatedAt,
}

#[derive(Iden)]
enum Posts {
    Table,
    Id,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
