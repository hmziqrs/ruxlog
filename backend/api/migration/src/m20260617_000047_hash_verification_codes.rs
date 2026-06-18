use sea_orm_migration::prelude::*;

/// Replace the plaintext `code` column on `email_verifications` and
/// `forgot_passwords` with `code_hash`, which stores `HMAC-SHA256(secret, code)`
/// (see `utils::code_hash`). Existing in-flight codes are plaintext and therefore
/// dropped — affected users simply re-request a code. This closes the
/// "brute-forceable plaintext reset codes" finding from the crypto audit.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // email_verifications: drop plaintext code, add code_hash.
        manager
            .alter_table(
                Table::alter()
                    .table(EmailVerifications::Table)
                    .drop_column(EmailVerifications::Code)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(EmailVerifications::Table)
                    .add_column(
                        ColumnDef::new(EmailVerifications::CodeHash)
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await?;

        // forgot_passwords: drop plaintext code, add code_hash.
        manager
            .alter_table(
                Table::alter()
                    .table(ForgotPasswords::Table)
                    .drop_column(ForgotPasswords::Code)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ForgotPasswords::Table)
                    .add_column(
                        ColumnDef::new(ForgotPasswords::CodeHash)
                            .string()
                            .not_null()
                            .default(""),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Restore the plaintext code columns (hashes cannot be reversed to codes,
        // so the down migration only restores schema shape, not data).
        manager
            .alter_table(
                Table::alter()
                    .table(EmailVerifications::Table)
                    .drop_column(EmailVerifications::CodeHash)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(EmailVerifications::Table)
                    .add_column(ColumnDef::new(EmailVerifications::Code).string().not_null().default(""))
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(ForgotPasswords::Table)
                    .drop_column(ForgotPasswords::CodeHash)
                    .to_owned(),
            )
            .await?;
        manager
            .alter_table(
                Table::alter()
                    .table(ForgotPasswords::Table)
                    .add_column(ColumnDef::new(ForgotPasswords::Code).string().not_null().default(""))
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
enum EmailVerifications {
    Table,
    Code,
    CodeHash,
}

#[derive(Iden)]
enum ForgotPasswords {
    Table,
    Code,
    CodeHash,
}
