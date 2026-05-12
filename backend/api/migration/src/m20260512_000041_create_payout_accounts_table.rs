use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(PayoutAccounts::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PayoutAccounts::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(PayoutAccounts::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(PayoutAccounts::Provider)
                            .string_len(50)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PayoutAccounts::ProviderAccountId)
                            .string_len(255)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PayoutAccounts::Status)
                            .string_len(20)
                            .not_null()
                            .default("pending"),
                    )
                    .col(ColumnDef::new(PayoutAccounts::Metadata).json_binary())
                    .col(
                        ColumnDef::new(PayoutAccounts::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(PayoutAccounts::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_payout_accounts_user_id")
                            .from(PayoutAccounts::Table, PayoutAccounts::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique index on user_id
        manager
            .create_index(
                Index::create()
                    .name("idx_payout_accounts_user_id_unique")
                    .table(PayoutAccounts::Table)
                    .col(PayoutAccounts::UserId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Composite index on provider + provider_account_id
        manager
            .create_index(
                Index::create()
                    .name("idx_payout_accounts_provider_account_id")
                    .table(PayoutAccounts::Table)
                    .col(PayoutAccounts::Provider)
                    .col(PayoutAccounts::ProviderAccountId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PayoutAccounts::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum PayoutAccounts {
    Table,
    Id,
    UserId,
    Provider,
    ProviderAccountId,
    Status,
    Metadata,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
