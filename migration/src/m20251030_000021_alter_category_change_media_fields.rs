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
                    .drop_column(Alias::new("cover_image"))
                    .drop_column(Alias::new("logo_image"))
                    .add_column(ColumnDef::new(Categories::CoverId).integer().null())
                    .add_column(ColumnDef::new(Categories::LogoId).integer().null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_category_cover")
                    .from(Categories::Table, Categories::CoverId)
                    .to(Media::Table, Media::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_category_logo")
                    .from(Categories::Table, Categories::LogoId)
                    .to(Media::Table, Media::Id)
                    .on_delete(ForeignKeyAction::SetNull)
                    .on_update(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_category_cover")
                    .table(Categories::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_category_logo")
                    .table(Categories::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Categories::Table)
                    .drop_column(Alias::new("cover_id"))
                    .drop_column(Alias::new("logo_id"))
                    .add_column(ColumnDef::new(Categories::CoverImage).string().null())
                    .add_column(ColumnDef::new(Categories::LogoImage).string().null())
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Categories {
    Table,
    CoverId,
    LogoId,
    CoverImage,
    LogoImage,
}

#[derive(Iden)]
enum Media {
    Table,
    Id,
}
