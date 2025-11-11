use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column(ColumnDef::new(Users::GoogleId).string().unique_key())
                    .add_column(ColumnDef::new(Users::OauthProvider).string())
                    .to_owned(),
            )
            .await?;

        // Make password nullable since OAuth users won't have passwords
        manager
            .get_connection()
            .execute_unprepared(r#"ALTER TABLE users ALTER COLUMN password DROP NOT NULL;"#)
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Make password not null again
        manager
            .get_connection()
            .execute_unprepared(r#"ALTER TABLE users ALTER COLUMN password SET NOT NULL;"#)
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::GoogleId)
                    .drop_column(Users::OauthProvider)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Users {
    Table,
    GoogleId,
    OauthProvider,
}
