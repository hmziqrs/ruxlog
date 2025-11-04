use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(RouteStatus::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RouteStatus::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(RouteStatus::RoutePattern).string().not_null().unique_key())
                    .col(ColumnDef::new(RouteStatus::IsBlocked).boolean().not_null().default(false))
                    .col(ColumnDef::new(RouteStatus::Reason).string().null())
                    .col(ColumnDef::new(RouteStatus::CreatedAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(RouteStatus::UpdatedAt).timestamp_with_time_zone().not_null())
                    .index(Index::create().name("idx_route_status_pattern").col(RouteStatus::RoutePattern))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(RouteStatus::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum RouteStatus {
    Table,
    Id,
    RoutePattern,
    IsBlocked,
    Reason,
    CreatedAt,
    UpdatedAt,
}