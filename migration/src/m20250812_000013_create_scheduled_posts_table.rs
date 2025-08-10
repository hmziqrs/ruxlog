use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_query::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Creates PostgreSQL enum `scheduled_post_status` and table `scheduled_posts`:
/// - id (pk)
/// - post_id -> posts.id (FK, cascade on delete/update)
/// - publish_at (timestamptz)
/// - status (scheduled_post_status)
/// - created_at (timestamptz)
/// - updated_at (timestamptz)
///
/// Indexes:
/// - idx_scheduled_posts_post_id (post_id)
/// - idx_scheduled_posts_status_publish_at (status, publish_at)
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create enum type for scheduled post status
        manager
            .create_type(
                Type::create()
                    .as_enum(ScheduledPostStatus::Table)
                    .values(vec![
                        ScheduledPostStatus::Pending,
                        ScheduledPostStatus::Published,
                        ScheduledPostStatus::Canceled,
                        ScheduledPostStatus::Failed,
                    ])
                    .to_owned(),
            )
            .await?;

        // Create scheduled_posts table
        manager
            .create_table(
                Table::create()
                    .table(ScheduledPosts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ScheduledPosts::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ScheduledPosts::PostId).integer().not_null())
                    .col(
                        ColumnDef::new(ScheduledPosts::PublishAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ScheduledPosts::Status)
                            .enumeration(
                                ScheduledPostStatus::Table,
                                [
                                    ScheduledPostStatus::Pending,
                                    ScheduledPostStatus::Published,
                                    ScheduledPostStatus::Canceled,
                                    ScheduledPostStatus::Failed,
                                ],
                            )
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ScheduledPosts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ScheduledPosts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_scheduled_posts_post")
                            .from(ScheduledPosts::Table, ScheduledPosts::PostId)
                            .to(Posts::Table, Posts::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Index on post_id
        manager
            .create_index(
                Index::create()
                    .name("idx_scheduled_posts_post_id")
                    .table(ScheduledPosts::Table)
                    .col(ScheduledPosts::PostId)
                    .to_owned(),
            )
            .await?;

        // Composite index for efficient due lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_scheduled_posts_status_publish_at")
                    .table(ScheduledPosts::Table)
                    .col(ScheduledPosts::Status)
                    .col(ScheduledPosts::PublishAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop table first, then enum type
        manager
            .drop_table(Table::drop().table(ScheduledPosts::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(ScheduledPostStatus::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum ScheduledPosts {
    Table,
    Id,
    PostId,
    PublishAt,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Posts {
    Table,
    Id,
}

#[derive(Iden)]
enum ScheduledPostStatus {
    Table,
    #[iden = "pending"]
    Pending,
    #[iden = "published"]
    Published,
    #[iden = "canceled"]
    Canceled,
    #[iden = "failed"]
    Failed,
}
