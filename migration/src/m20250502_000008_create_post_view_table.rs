use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PostViews::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PostViews::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PostViews::PostId).integer().not_null())
                    .col(ColumnDef::new(PostViews::UserId).integer())
                    .col(ColumnDef::new(PostViews::IpAddress).string())
                    .col(ColumnDef::new(PostViews::UserAgent).string())
                    .col(
                        ColumnDef::new(PostViews::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_views_post")
                            .from(PostViews::Table, PostViews::PostId)
                            .to(Posts::Table, Posts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_views_user")
                            .from(PostViews::Table, PostViews::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PostViews::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum PostViews {
    Table,
    Id,
    PostId,
    UserId,
    IpAddress,
    UserAgent,
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
