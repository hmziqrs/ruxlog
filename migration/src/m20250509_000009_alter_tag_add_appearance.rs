use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Tags::Table)
                    .add_column(
                        ColumnDef::new(Tags::Color)
                            .string()
                            .not_null()
                            .default("#3b82f6"),
                    )
                    .add_column(
                        ColumnDef::new(Tags::TextColor)
                            .string()
                            .not_null()
                            .default("#111111"),
                    )
                    .add_column(
                        ColumnDef::new(Tags::IsActive)
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
                    .table(Tags::Table)
                    .drop_column(Tags::Color)
                    .drop_column(Tags::TextColor)
                    .drop_column(Tags::IsActive)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Tags {
    Table,
    Id,
    Name,
    Slug,
    Description,
    Color,
    TextColor,
    IsActive,
    CreatedAt,
    UpdatedAt,
}
