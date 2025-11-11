use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Media::Table)
                    .add_column(
                        ColumnDef::new(Media::ContentHash)
                            .string_len(128)
                            .unique_key()
                            .null(),
                    )
                    .add_column(
                        ColumnDef::new(Media::IsOptimized)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .add_column(
                        ColumnDef::new(Media::OptimizedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Media::Table)
                    .drop_column(Media::ContentHash)
                    .drop_column(Media::IsOptimized)
                    .drop_column(Media::OptimizedAt)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Media {
    Table,
    ContentHash,
    IsOptimized,
    OptimizedAt,
}
