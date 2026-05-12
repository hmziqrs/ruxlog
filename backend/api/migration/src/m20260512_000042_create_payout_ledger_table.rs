use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PayoutLedger::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PayoutLedger::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PayoutLedger::UserId).integer().not_null())
                    .col(ColumnDef::new(PayoutLedger::AmountCents).integer().not_null())
                    .col(
                        ColumnDef::new(PayoutLedger::Currency)
                            .string_len(3)
                            .not_null()
                            .default("USD"),
                    )
                    .col(
                        ColumnDef::new(PayoutLedger::EntryType)
                            .string_len(20)
                            .not_null(),
                    )
                    .col(ColumnDef::new(PayoutLedger::ReferenceType).string_len(50))
                    .col(ColumnDef::new(PayoutLedger::ReferenceId).string_len(255))
                    .col(ColumnDef::new(PayoutLedger::Description).text())
                    .col(ColumnDef::new(PayoutLedger::BalanceAfter).integer().not_null())
                    .col(
                        ColumnDef::new(PayoutLedger::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_payout_ledger_user_id")
                            .from(PayoutLedger::Table, PayoutLedger::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Composite index on user_id + created_at
        manager
            .create_index(
                Index::create()
                    .name("idx_payout_ledger_user_created")
                    .table(PayoutLedger::Table)
                    .col(PayoutLedger::UserId)
                    .col(PayoutLedger::CreatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PayoutLedger::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum PayoutLedger {
    Table,
    Id,
    UserId,
    AmountCents,
    Currency,
    EntryType,
    ReferenceType,
    ReferenceId,
    Description,
    BalanceAfter,
    CreatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
