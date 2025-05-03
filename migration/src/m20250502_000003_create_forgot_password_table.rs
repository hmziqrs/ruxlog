use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ForgotPasswords::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ForgotPasswords::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(ForgotPasswords::UserId).integer().not_null())
                    .col(ColumnDef::new(ForgotPasswords::Code).string().not_null())
                    .col(
                        ColumnDef::new(ForgotPasswords::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ForgotPasswords::ExpiresAt)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_forgot_passwords_user")
                            .from(ForgotPasswords::Table, ForgotPasswords::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(ForgotPasswords::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum ForgotPasswords {
    Table,
    Id,
    UserId,
    Code,
    CreatedAt,
    ExpiresAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
