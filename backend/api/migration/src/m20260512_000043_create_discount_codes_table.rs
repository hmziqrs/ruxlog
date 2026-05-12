use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(DiscountCodes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(DiscountCodes::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(DiscountCodes::Code).string_len(50).not_null())
                    .col(ColumnDef::new(DiscountCodes::Description).text())
                    .col(
                        ColumnDef::new(DiscountCodes::DiscountType)
                            .string_len(20)
                            .not_null(),
                    )
                    .col(ColumnDef::new(DiscountCodes::DiscountValue).integer().not_null())
                    .col(
                        ColumnDef::new(DiscountCodes::Currency)
                            .string_len(3)
                            .default("USD"),
                    )
                    .col(ColumnDef::new(DiscountCodes::MaxRedemptions).integer())
                    .col(
                        ColumnDef::new(DiscountCodes::RedeemedCount)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(DiscountCodes::ValidFrom).timestamp_with_time_zone(),
                    )
                    .col(ColumnDef::new(DiscountCodes::ValidUntil).timestamp_with_time_zone())
                    .col(ColumnDef::new(DiscountCodes::PlanId).integer())
                    .col(
                        ColumnDef::new(DiscountCodes::IsActive)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(DiscountCodes::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(DiscountCodes::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_discount_codes_plan_id")
                            .from(DiscountCodes::Table, DiscountCodes::PlanId)
                            .to(Plans::Table, Plans::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique index on code
        manager
            .create_index(
                Index::create()
                    .name("idx_discount_codes_code_unique")
                    .table(DiscountCodes::Table)
                    .col(DiscountCodes::Code)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Index on is_active
        manager
            .create_index(
                Index::create()
                    .name("idx_discount_codes_is_active")
                    .table(DiscountCodes::Table)
                    .col(DiscountCodes::IsActive)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(DiscountCodes::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum DiscountCodes {
    Table,
    Id,
    Code,
    Description,
    DiscountType,
    DiscountValue,
    Currency,
    MaxRedemptions,
    RedeemedCount,
    ValidFrom,
    ValidUntil,
    PlanId,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Plans {
    Table,
    Id,
}
