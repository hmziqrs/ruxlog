use sea_orm_migration::prelude::{extension::postgres::Type, *};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create post_status enum type

        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE TYPE "post_status" AS ENUM ('draft', 'published', 'archived');"#,
            )
            .await?;

        // Create posts table
        manager
            .create_table(
                Table::create()
                    .table(Posts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Posts::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Posts::Title).string().not_null())
                    .col(ColumnDef::new(Posts::Slug).string().not_null().unique_key())
                    .col(ColumnDef::new(Posts::Content).text().not_null())
                    .col(ColumnDef::new(Posts::Excerpt).text())
                    .col(ColumnDef::new(Posts::FeaturedImage).string())
                    .col(
                        ColumnDef::new(Posts::Status) // Changed from PostStatus to Status
                            .custom(PostStatus::PostStatus) // Use the Iden for the custom type
                            .not_null()
                            .default(Expr::val("draft")),

                    )
                    .col(ColumnDef::new(Posts::PublishedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Posts::AuthorId).integer().not_null())
                    .col(ColumnDef::new(Posts::CategoryId).integer().not_null())
                    .col(
                        ColumnDef::new(Posts::ViewCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Posts::LikesCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(Posts::TagIds)
                            .array(ColumnType::Integer)
                            .not_null()
                            .default(Expr::cust("'{}'::integer[]")),
                    )
                    .col(
                        ColumnDef::new(Posts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Posts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_posts_author")
                            .from(Posts::Table, Posts::AuthorId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_posts_category")
                            .from(Posts::Table, Posts::CategoryId)
                            .to(Categories::Table, Categories::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Posts::Table).to_owned())
            .await?;

        manager
            .get_connection()
            .execute_unprepared(r#"DROP TYPE "post_status";"#) // Corrected type name
            .await?;

        Ok(())

    }
}

#[derive(Iden)]
enum Posts {
    Table,
    Id,
    Title,
    Slug,
    Content,
    Excerpt,
    FeaturedImage,
    Status,
    PublishedAt,
    AuthorId,
    CategoryId,
    ViewCount,
    LikesCount,
    TagIds,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum PostStatus {
    #[iden = "post_status"]
    PostStatus,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}

#[derive(Iden)]
enum Categories {
    Table,
    Id,
}
