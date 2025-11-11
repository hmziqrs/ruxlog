use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Creates post series structures:
/// - post_series (id, name, slug UNIQUE, description, created_at, updated_at)
/// - post_series_posts (id, series_id FK, post_id FK, sort_order, created_at, updated_at)
///
/// Indexes:
/// - uniq_post_series_slug (unique on slug)
/// - idx_post_series_posts_series_id
/// - idx_post_series_posts_post_id
/// - idx_post_series_posts_series_sort (series_id, sort_order) for ordered listing
/// - uniq_post_series_posts_series_post (unique on (series_id, post_id))
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // post_series
        manager
            .create_table(
                Table::create()
                    .table(PostSeries::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PostSeries::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PostSeries::Name).string().not_null())
                    .col(ColumnDef::new(PostSeries::Slug).string().not_null())
                    .col(ColumnDef::new(PostSeries::Description).text().null())
                    .col(
                        ColumnDef::new(PostSeries::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PostSeries::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique slug for series
        manager
            .create_index(
                Index::create()
                    .name("uniq_post_series_slug")
                    .table(PostSeries::Table)
                    .col(PostSeries::Slug)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // post_series_posts (mapping table)
        manager
            .create_table(
                Table::create()
                    .table(PostSeriesPosts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PostSeriesPosts::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PostSeriesPosts::SeriesId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(PostSeriesPosts::PostId).integer().not_null())
                    .col(
                        ColumnDef::new(PostSeriesPosts::SortOrder)
                            .integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PostSeriesPosts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PostSeriesPosts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_series_posts_series")
                            .from(PostSeriesPosts::Table, PostSeriesPosts::SeriesId)
                            .to(PostSeries::Table, PostSeries::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_series_posts_post")
                            .from(PostSeriesPosts::Table, PostSeriesPosts::PostId)
                            .to(Posts::Table, Posts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique mapping per (series_id, post_id)
        manager
            .create_index(
                Index::create()
                    .name("uniq_post_series_posts_series_post")
                    .table(PostSeriesPosts::Table)
                    .col(PostSeriesPosts::SeriesId)
                    .col(PostSeriesPosts::PostId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Indexes to speed up lookups and ordering
        manager
            .create_index(
                Index::create()
                    .name("idx_post_series_posts_series_id")
                    .table(PostSeriesPosts::Table)
                    .col(PostSeriesPosts::SeriesId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_post_series_posts_post_id")
                    .table(PostSeriesPosts::Table)
                    .col(PostSeriesPosts::PostId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_post_series_posts_series_sort")
                    .table(PostSeriesPosts::Table)
                    .col(PostSeriesPosts::SeriesId)
                    .col(PostSeriesPosts::SortOrder)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop mapping table first due to FKs
        manager
            .drop_table(Table::drop().table(PostSeriesPosts::Table).to_owned())
            .await?;

        // Drop series table
        manager
            .drop_table(Table::drop().table(PostSeries::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum PostSeries {
    Table,
    Id,
    Name,
    Slug,
    Description,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum PostSeriesPosts {
    Table,
    Id,
    SeriesId,
    PostId,
    SortOrder,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Posts {
    Table,
    Id,
}
