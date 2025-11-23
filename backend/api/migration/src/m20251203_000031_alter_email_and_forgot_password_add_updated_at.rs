use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(EmailVerifications::Table)
                    .add_column(
                        ColumnDef::new(EmailVerifications::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .drop_column(EmailVerifications::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ForgotPasswords::Table)
                    .add_column(
                        ColumnDef::new(ForgotPasswords::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .drop_column(ForgotPasswords::ExpiresAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(EmailVerifications::Table)
                    .add_column(
                        ColumnDef::new(EmailVerifications::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .drop_column(EmailVerifications::UpdatedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ForgotPasswords::Table)
                    .add_column(
                        ColumnDef::new(ForgotPasswords::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .drop_column(ForgotPasswords::UpdatedAt)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum EmailVerifications {
    Table,
    ExpiresAt,
    UpdatedAt,
}

#[derive(Iden)]
enum ForgotPasswords {
    Table,
    ExpiresAt,
    UpdatedAt,
}
