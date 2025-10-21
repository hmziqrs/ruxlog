use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_query::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Creates the `media` table for storing public S3 assets with loose associations.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create enum type for describing the high-level owner of the media file
        manager
            .create_type(
                Type::create()
                    .as_enum(MediaReferenceType::Table)
                    .values(vec![
                        MediaReferenceType::Category,
                        MediaReferenceType::User,
                        MediaReferenceType::Post,
                    ])
                    .to_owned(),
            )
            .await?;

        // Create the media table
        manager
            .create_table(
                Table::create()
                    .table(Media::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Media::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Media::ObjectKey).text().not_null())
                    .col(ColumnDef::new(Media::FileUrl).text().not_null())
                    .col(ColumnDef::new(Media::MimeType).text().not_null())
                    .col(ColumnDef::new(Media::Width).integer())
                    .col(ColumnDef::new(Media::Height).integer())
                    .col(ColumnDef::new(Media::Size).big_integer().not_null())
                    .col(ColumnDef::new(Media::Extension).string_len(16))
                    .col(ColumnDef::new(Media::UploaderId).integer())
                    .col(
                        ColumnDef::new(Media::ReferenceType)
                            .enumeration(
                                MediaReferenceType::Table,
                                [
                                    MediaReferenceType::Category,
                                    MediaReferenceType::User,
                                    MediaReferenceType::Post,
                                ],
                            )
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Media::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Media::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_media_uploader")
                            .from(Media::Table, Media::UploaderId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Helpful indexes for frequent lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_media_uploader")
                    .table(Media::Table)
                    .col(Media::UploaderId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_media_reference_type")
                    .table(Media::Table)
                    .col(Media::ReferenceType)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_media_reference_type")
                    .table(Media::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_media_uploader")
                    .table(Media::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Media::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(MediaReferenceType::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Media {
    Table,
    Id,
    ObjectKey,
    FileUrl,
    MimeType,
    Width,
    Height,
    Size,
    Extension,
    UploaderId,
    ReferenceType,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}

#[derive(Iden)]
enum MediaReferenceType {
    Table,
    #[iden = "category"]
    Category,
    #[iden = "user"]
    User,
    #[iden = "post"]
    Post,
}
