use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PostComments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PostComments::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PostComments::PostId).integer().not_null())
                    .col(ColumnDef::new(PostComments::UserId).integer().not_null())
                    .col(ColumnDef::new(PostComments::Content).text().not_null())
                    .col(
                        ColumnDef::new(PostComments::LikesCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(PostComments::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(PostComments::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_comments_post")
                            .from(PostComments::Table, PostComments::PostId)
                            .to(Posts::Table, Posts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_comments_user")
                            .from(PostComments::Table, PostComments::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PostComments::Table).to_owned())
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
