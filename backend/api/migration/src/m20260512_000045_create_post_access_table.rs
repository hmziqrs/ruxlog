use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PostAccess::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PostAccess::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PostAccess::PostId).integer().not_null())
                    .col(
                        ColumnDef::new(PostAccess::AccessType)
                            .string_len(20)
                            .not_null()
                            .default("free"),
                    )
                    .col(ColumnDef::new(PostAccess::PriceCents).integer())
                    .col(
                        ColumnDef::new(PostAccess::Currency)
                            .string_len(3)
                            .default("USD"),
                    )
                    .col(
                        ColumnDef::new(PostAccess::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_post_access_post_id")
                            .from(PostAccess::Table, PostAccess::PostId)
                            .to(Posts::Table, Posts::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique index: one access rule per post
        manager
            .create_index(
                Index::create()
                    .name("idx_post_access_post_id_unique")
                    .table(PostAccess::Table)
                    .col(PostAccess::PostId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PostAccess::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum PostAccess {
    Table,
    Id,
    PostId,
    AccessType,
    PriceCents,
    Currency,
    CreatedAt,
}

#[derive(Iden)]
enum Posts {
    Table,
    Id,
}
