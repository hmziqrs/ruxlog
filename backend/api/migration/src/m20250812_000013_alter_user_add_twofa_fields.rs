use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum Users {
    Table,
    TwoFaEnabled,
    TwoFaSecret,
    TwoFaBackupCodes,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add 2FA fields to users table:
        // - two_fa_enabled: bool NOT NULL DEFAULT false
        // - two_fa_secret: text NULL
        // - two_fa_backup_codes: jsonb NULL
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(
                        ColumnDef::new(Users::TwoFaEnabled)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .add_column(ColumnDef::new(Users::TwoFaSecret).text().null())
                    .add_column(ColumnDef::new(Users::TwoFaBackupCodes).json_binary().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove the added 2FA fields (reverse order recommended)
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::TwoFaBackupCodes)
                    .drop_column(Users::TwoFaSecret)
                    .drop_column(Users::TwoFaEnabled)
                    .to_owned(),
            )
            .await
    }
}
