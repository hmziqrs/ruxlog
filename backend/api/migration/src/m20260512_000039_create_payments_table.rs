use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Payments::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Payments::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Payments::UserId).integer().not_null())
                    .col(ColumnDef::new(Payments::SubscriptionId).integer())
                    .col(ColumnDef::new(Payments::PlanId).integer())
                    .col(ColumnDef::new(Payments::Provider).string_len(50).not_null())
                    .col(ColumnDef::new(Payments::ProviderPaymentId).string_len(255))
                    .col(ColumnDef::new(Payments::AmountCents).integer().not_null())
                    .col(
                        ColumnDef::new(Payments::Currency)
                            .string_len(3)
                            .not_null()
                            .default("USD"),
                    )
                    .col(
                        ColumnDef::new(Payments::Status)
                            .string_len(20)
                            .not_null()
                            .default("pending"),
                    )
                    .col(ColumnDef::new(Payments::Description).text())
                    .col(ColumnDef::new(Payments::Metadata).json_binary())
                    .col(
                        ColumnDef::new(Payments::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Payments::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_payments_user_id")
                            .from(Payments::Table, Payments::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_payments_subscription_id")
                            .from(Payments::Table, Payments::SubscriptionId)
                            .to(Subscriptions::Table, Subscriptions::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_payments_plan_id")
                            .from(Payments::Table, Payments::PlanId)
                            .to(Plans::Table, Plans::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Index on user_id
        manager
            .create_index(
                Index::create()
                    .name("idx_payments_user_id")
                    .table(Payments::Table)
                    .col(Payments::UserId)
                    .to_owned(),
            )
            .await?;

        // Unique index on provider + provider_payment_id
        manager
            .create_index(
                Index::create()
                    .name("idx_payments_provider_payment_id_unique")
                    .table(Payments::Table)
                    .col(Payments::Provider)
                    .col(Payments::ProviderPaymentId)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Payments::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Payments {
    Table,
    Id,
    UserId,
    SubscriptionId,
    PlanId,
    Provider,
    ProviderPaymentId,
    AmountCents,
    Currency,
    Status,
    Description,
    Metadata,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}

#[derive(Iden)]
enum Subscriptions {
    Table,
    Id,
}

#[derive(Iden)]
enum Plans {
    Table,
    Id,
}
