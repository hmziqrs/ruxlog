use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Assets::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Assets::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Assets::FileUrl).text().not_null())
                    .col(ColumnDef::new(Assets::FileName).text())
                    .col(ColumnDef::new(Assets::MimeType).text())
                    .col(ColumnDef::new(Assets::Size).integer())
                    .col(
                        ColumnDef::new(Assets::UploadedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(Assets::OwnerId).integer())
                    .col(ColumnDef::new(Assets::Context).text())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_assets_user")
                            .from(Assets::Table, Assets::OwnerId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Assets::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Assets {
    Table,
    Id,
    FileUrl,
    FileName,
    MimeType,
    Size,
    UploadedAt,
    OwnerId,
    Context,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}