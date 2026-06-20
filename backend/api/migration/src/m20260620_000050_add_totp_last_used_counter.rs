use sea_orm_migration::prelude::*;

/// Adds a nullable `two_fa_last_totp_counter` column to `users`.
///
/// V-MED-6 (TOTP replay): TOTP verification is otherwise a stateless pure
/// function — an accepted 6-digit code stays replayable for ~90s
/// (window=1 → 3 steps of 30s). This column stores the highest RFC 6238 time
/// step counter whose code was already accepted, so `twofa_verify` /
/// `twofa_disable` can reject a replayed code (`counter <= last_used`) and
/// only persist `counter > last_used`.
///
/// The column is nullable (default NULL): a `NULL` value means "no code has
/// been consumed yet", so first-time use accepts any valid in-window counter.
/// On disable we leave the counter in place; on a fresh setup it starts NULL.
#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum Users {
    Table,
    TwoFaLastTotpCounter,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::TwoFaLastTotpCounter)
                            .big_integer()
                            .null(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::TwoFaLastTotpCounter)
                    .to_owned(),
            )
            .await
    }
}
