use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

/// Moves `media.file_url` to a derived field by introducing `media.bucket` and relying on
/// `media.object_key` as the object key within that bucket.
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Bucket can be NULL for legacy rows; the application will fall back to using
        // `object_key` as a full path when bucket is missing.
        manager
            .alter_table(
                Table::alter()
                    .table(Media::Table)
                    .add_column(ColumnDef::new(Media::Bucket).text().null())
                    .drop_column(Media::FileUrl)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Re-introduce `file_url` with a harmless default; data cannot be reconstructed.
        manager
            .alter_table(
                Table::alter()
                    .table(Media::Table)
                    .add_column(ColumnDef::new(Media::FileUrl).text().not_null().default(""))
                    .drop_column(Media::Bucket)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum Media {
    Table,
    Bucket,
    FileUrl,
}
