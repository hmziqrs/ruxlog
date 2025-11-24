use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AppConstants::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AppConstants::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(AppConstants::Key)
                            .string_len(191)
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(AppConstants::Value).text().not_null())
                    .col(
                        ColumnDef::new(AppConstants::ValueType)
                            .string_len(50)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AppConstants::Description)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AppConstants::IsSensitive)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(AppConstants::Source)
                            .string_len(50)
                            .not_null()
                            .default("env"),
                    )
                    .col(
                        ColumnDef::new(AppConstants::UpdatedBy)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(AppConstants::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(AppConstants::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AppConstants::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AppConstants {
    Table,
    Id,
    Key,
    Value,
    ValueType,
    Description,
    IsSensitive,
    Source,
    UpdatedBy,
    CreatedAt,
    UpdatedAt,
}
