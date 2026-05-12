use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Invoices::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Invoices::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Invoices::UserId).integer().not_null())
                    .col(ColumnDef::new(Invoices::SubscriptionId).integer())
                    .col(ColumnDef::new(Invoices::PaymentId).integer())
                    .col(
                        ColumnDef::new(Invoices::InvoiceNumber)
                            .string_len(50)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Invoices::AmountCents).integer().not_null())
                    .col(
                        ColumnDef::new(Invoices::Currency)
                            .string_len(3)
                            .not_null()
                            .default("USD"),
                    )
                    .col(
                        ColumnDef::new(Invoices::Status)
                            .string_len(20)
                            .not_null()
                            .default("draft"),
                    )
                    .col(ColumnDef::new(Invoices::DueDate).timestamp_with_time_zone())
                    .col(ColumnDef::new(Invoices::PaidAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(Invoices::PdfUrl).text())
                    .col(ColumnDef::new(Invoices::Metadata).json_binary())
                    .col(
                        ColumnDef::new(Invoices::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Invoices::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_invoices_user_id")
                            .from(Invoices::Table, Invoices::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_invoices_subscription_id")
                            .from(Invoices::Table, Invoices::SubscriptionId)
                            .to(Subscriptions::Table, Subscriptions::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_invoices_payment_id")
                            .from(Invoices::Table, Invoices::PaymentId)
                            .to(Payments::Table, Payments::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Index on user_id
        manager
            .create_index(
                Index::create()
                    .name("idx_invoices_user_id")
                    .table(Invoices::Table)
                    .col(Invoices::UserId)
                    .to_owned(),
            )
            .await?;

        // Unique index on invoice_number
        manager
            .create_index(
                Index::create()
                    .name("idx_invoices_number_unique")
                    .table(Invoices::Table)
                    .col(Invoices::InvoiceNumber)
                    .unique()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Invoices::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Invoices {
    Table,
    Id,
    UserId,
    SubscriptionId,
    PaymentId,
    InvoiceNumber,
    AmountCents,
    Currency,
    Status,
    DueDate,
    PaidAt,
    PdfUrl,
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
enum Payments {
    Table,
    Id,
}
