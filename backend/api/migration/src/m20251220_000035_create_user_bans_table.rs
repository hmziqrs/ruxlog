use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(UserBans::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserBans::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserBans::UserId).integer().not_null())
                    .col(ColumnDef::new(UserBans::Reason).text())
                    .col(ColumnDef::new(UserBans::BannedBy).integer())
                    .col(
                        ColumnDef::new(UserBans::ExpiresAt)
                            .timestamp_with_time_zone(),
                    )
                    .col(
                        ColumnDef::new(UserBans::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(ColumnDef::new(UserBans::RevokedAt).timestamp_with_time_zone())
                    .col(ColumnDef::new(UserBans::RevokedBy).integer())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_bans_user_id")
                            .from(UserBans::Table, UserBans::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_bans_banned_by")
                            .from(UserBans::Table, UserBans::BannedBy)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_bans_revoked_by")
                            .from(UserBans::Table, UserBans::RevokedBy)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Index for looking up active bans by user
        manager
            .create_index(
                Index::create()
                    .name("idx_user_bans_user_id")
                    .table(UserBans::Table)
                    .col(UserBans::UserId)
                    .to_owned(),
            )
            .await?;

        // Partial index for active bans (not revoked, not expired or no expiry)
        manager
            .create_index(
                Index::create()
                    .name("idx_user_bans_active")
                    .table(UserBans::Table)
                    .col(UserBans::UserId)
                    .col(UserBans::ExpiresAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UserBans::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum UserBans {
    Table,
    Id,
    UserId,
    Reason,
    BannedBy,
    ExpiresAt,
    CreatedAt,
    RevokedAt,
    RevokedBy,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}
