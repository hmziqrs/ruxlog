use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Subscriptions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Subscriptions::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Subscriptions::UserId).integer().not_null())
                    .col(ColumnDef::new(Subscriptions::PlanId).integer().not_null())
                    .col(ColumnDef::new(Subscriptions::Provider).string_len(50).not_null())
                    .col(ColumnDef::new(Subscriptions::ProviderCustomerId).string_len(255))
                    .col(
                        ColumnDef::new(Subscriptions::ProviderSubscriptionId).string_len(255),
                    )
                    .col(
                        ColumnDef::new(Subscriptions::Status)
                            .string_len(30)
                            .not_null()
                            .default("active"),
                    )
                    .col(
                        ColumnDef::new(Subscriptions::CurrentPeriodStart)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(Subscriptions::CurrentPeriodEnd)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(Subscriptions::CancelAtPeriodEnd)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(ColumnDef::new(Subscriptions::TrialEndsAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Subscriptions::Metadata).json_binary())
                    .col(
                        ColumnDef::new(Subscriptions::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Subscriptions::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_subscriptions_user_id")
                            .from(Subscriptions::Table, Subscriptions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_subscriptions_plan_id")
                            .from(Subscriptions::Table, Subscriptions::PlanId)
                            .to(Plans::Table, Plans::Id)
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        // Index on user_id
        manager
            .create_index(
                Index::create()
                    .name("idx_subscriptions_user_id")
                    .table(Subscriptions::Table)
                    .col(Subscriptions::UserId)
                    .to_owned(),
            )
            .await?;

        // Composite index on provider + provider_subscription_id
        manager
            .create_index(
                Index::create()
                    .name("idx_subscriptions_provider_sub_id")
                    .table(Subscriptions::Table)
                    .col(Subscriptions::Provider)
                    .col(Subscriptions::ProviderSubscriptionId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Subscriptions::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Subscriptions {
    Table,
    Id,
    UserId,
    PlanId,
    Provider,
    ProviderCustomerId,
    ProviderSubscriptionId,
    Status,
    CurrentPeriodStart,
    CurrentPeriodEnd,
    CancelAtPeriodEnd,
    TrialEndsAt,
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
enum Plans {
    Table,
    Id,
}
