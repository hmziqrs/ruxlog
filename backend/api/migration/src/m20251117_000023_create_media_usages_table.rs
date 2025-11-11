use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_query::extension::postgres::Type;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(EntityTypeEnum::Enum)
                    .values([
                        EntityTypeEnum::Category,
                        EntityTypeEnum::User,
                        EntityTypeEnum::Post,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(MediaUsage::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(MediaUsage::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(MediaUsage::MediaId).integer().not_null())
                    .col(
                        ColumnDef::new(MediaUsage::EntityType)
                            .enumeration(
                                EntityTypeEnum::Enum,
                                [
                                    EntityTypeEnum::Category,
                                    EntityTypeEnum::User,
                                    EntityTypeEnum::Post,
                                ],
                            )
                            .not_null(),
                    )
                    .col(ColumnDef::new(MediaUsage::EntityId).integer().not_null())
                    .col(
                        ColumnDef::new(MediaUsage::FieldName)
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(MediaUsage::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_media_usage_media")
                            .from(MediaUsage::Table, MediaUsage::MediaId)
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
                    .name("idx_media_usage_media_id")
                    .table(MediaUsage::Table)
                    .col(MediaUsage::MediaId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_media_usage_entity")
                    .table(MediaUsage::Table)
                    .col(MediaUsage::EntityType)
                    .col(MediaUsage::EntityId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("uniq_media_usage_composite")
                    .table(MediaUsage::Table)
                    .col(MediaUsage::MediaId)
                    .col(MediaUsage::EntityType)
                    .col(MediaUsage::EntityId)
                    .col(MediaUsage::FieldName)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MediaUsage::Table).to_owned())
            .await?;

        manager
            .drop_type(Type::drop().name(EntityTypeEnum::Enum).to_owned())
            .await
    }
}

#[derive(Iden)]
enum MediaUsage {
    Table,
    Id,
    MediaId,
    EntityType,
    EntityId,
    FieldName,
    CreatedAt,
}

#[derive(Iden)]
enum Media {
    Table,
    Id,
}

#[derive(Iden)]
enum EntityTypeEnum {
    #[iden = "entity_type"]
    Enum,
    #[iden = "category"]
    Category,
    #[iden = "user"]
    User,
    #[iden = "post"]
    Post,
}
