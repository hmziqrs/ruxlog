use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Plans::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Plans::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Plans::Name).string_len(255).not_null())
                    .col(ColumnDef::new(Plans::Slug).string_len(255).not_null())
                    .col(ColumnDef::new(Plans::Description).text())
                    .col(ColumnDef::new(Plans::PriceCents).integer().not_null())
                    .col(
                        ColumnDef::new(Plans::Currency)
                            .string_len(3)
                            .not_null()
                            .default("USD"),
                    )
                    .col(ColumnDef::new(Plans::Interval).string_len(10).not_null())
                    .col(ColumnDef::new(Plans::TrialDays).integer().default(0))
                    .col(ColumnDef::new(Plans::Features).json_binary())
                    .col(
                        ColumnDef::new(Plans::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(ColumnDef::new(Plans::SortOrder).integer().not_null().default(0))
                    .col(
                        ColumnDef::new(Plans::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Plans::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique index on slug
        manager
            .create_index(
                Index::create()
                    .name("idx_plans_slug_unique")
                    .table(Plans::Table)
                    .col(Plans::Slug)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Index on is_active
        manager
            .create_index(
                Index::create()
                    .name("idx_plans_is_active")
                    .table(Plans::Table)
                    .col(Plans::IsActive)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Plans::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Plans {
    Table,
    Id,
    Name,
    Slug,
    Description,
    PriceCents,
    Currency,
    Interval,
    TrialDays,
    Features,
    IsActive,
    SortOrder,
    CreatedAt,
    UpdatedAt,
}
