use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum NewsletterSubscribers {
    Table,
    Id,
    Email,
    Status,
    Token,
    CreatedAt,
    UpdatedAt,
}

impl NewsletterSubscribers {
    fn table() -> Self {
        Self::Table
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create newsletter_subscribers table
        manager
            .create_table(
                Table::create()
                    .table(NewsletterSubscribers::table())
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NewsletterSubscribers::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(NewsletterSubscribers::Email)
                            .string_len(320) // RFC 3696/5321 practical max
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NewsletterSubscribers::Status)
                            .string_len(32) // "pending" | "confirmed" | "unsubscribed"
                            .not_null()
                            .default("pending"),
                    )
                    .col(
                        ColumnDef::new(NewsletterSubscribers::Token)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NewsletterSubscribers::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NewsletterSubscribers::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uq_newsletter_subscribers_email")
                    .table(NewsletterSubscribers::Table)
                    .col(NewsletterSubscribers::Email)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_newsletter_subscribers_status")
                    .table(NewsletterSubscribers::Table)
                    .col(NewsletterSubscribers::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_newsletter_subscribers_created_at")
                    .table(NewsletterSubscribers::Table)
                    .col(NewsletterSubscribers::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop newsletter_subscribers table
        manager
            .drop_table(
                Table::drop()
                    .table(NewsletterSubscribers::table())
                    .to_owned(),
            )
            .await
    }
}
