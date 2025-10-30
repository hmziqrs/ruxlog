use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MediaVariant::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MediaVariant::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(MediaVariant::MediaId).integer().not_null())
                    .col(ColumnDef::new(MediaVariant::ObjectKey).text().not_null())
                    .col(ColumnDef::new(MediaVariant::MimeType).text().not_null())
                    .col(ColumnDef::new(MediaVariant::Width).integer())
                    .col(ColumnDef::new(MediaVariant::Height).integer())
                    .col(ColumnDef::new(MediaVariant::Size).big_integer().not_null())
                    .col(ColumnDef::new(MediaVariant::Extension).string_len(16))
                    .col(ColumnDef::new(MediaVariant::Quality).integer())
                    .col(
                        ColumnDef::new(MediaVariant::VariantType)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MediaVariant::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(MediaVariant::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_media_variant_media")
                            .from(MediaVariant::Table, MediaVariant::MediaId)
                            .to(Media::Table, Media::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_media_variant_media_id")
                    .table(MediaVariant::Table)
                    .col(MediaVariant::MediaId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_media_variant_variant_type")
                    .table(MediaVariant::Table)
                    .col(MediaVariant::VariantType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uniq_media_variant_object_key")
                    .table(MediaVariant::Table)
                    .col(MediaVariant::ObjectKey)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MediaVariant::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum MediaVariant {
    Table,
    Id,
    MediaId,
    ObjectKey,
    MimeType,
    Width,
    Height,
    Size,
    Extension,
    Quality,
    VariantType,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Media {
    Table,
    Id,
}
