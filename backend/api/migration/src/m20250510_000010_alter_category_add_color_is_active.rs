use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Categories::Table)
                    .add_column(
                        ColumnDef::new(Categories::Color)
                            .string()
                            .not_null()
                            .default("#3b82f6"),
                    )
                    .add_column(
                        ColumnDef::new(Categories::TextColor)
                            .string()
                            .not_null()
                            .default("#111111"),
                    )
                    .add_column(
                        ColumnDef::new(Categories::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Categories::Table)
                    .drop_column(Categories::Color)
                    .drop_column(Categories::TextColor)
                    .drop_column(Categories::IsActive)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Categories {
    Table,
    Color,
    TextColor,
    IsActive,
}
