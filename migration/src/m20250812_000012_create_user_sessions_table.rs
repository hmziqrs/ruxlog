use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(Iden)]
enum UserSessions {
    Table,
    Id,
    UserId,
    Device,
    IpAddress,
    LastSeen,
    RevokedAt,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create user_sessions table
        manager
            .create_table(
                Table::create()
                    .table(UserSessions::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(UserSessions::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserSessions::UserId).integer().not_null())
                    .col(ColumnDef::new(UserSessions::Device).string().null())
                    .col(ColumnDef::new(UserSessions::IpAddress).string().null())
                    .col(
                        ColumnDef::new(UserSessions::LastSeen)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(UserSessions::RevokedAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_sessions_user_id")
                            .from(UserSessions::Table, UserSessions::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Indexes for query performance
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_user_sessions_user_id")
                    .table(UserSessions::Table)
                    .col(UserSessions::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_user_sessions_user_id_revoked_at")
                    .table(UserSessions::Table)
                    .col(UserSessions::UserId)
                    .col(UserSessions::RevokedAt)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_user_sessions_last_seen")
                    .table(UserSessions::Table)
                    .col(UserSessions::LastSeen)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop indexes first (optional; dropping table typically cascades index removal)
        manager
            .drop_index(
                Index::drop()
                    .name("idx_user_sessions_last_seen")
                    .table(UserSessions::Table)
                    .to_owned(),
            )
            .await
            .ok(); // Ignore if missing

        manager
            .drop_index(
                Index::drop()
                    .name("idx_user_sessions_user_id_revoked_at")
                    .table(UserSessions::Table)
                    .to_owned(),
            )
            .await
            .ok();

        manager
            .drop_index(
                Index::drop()
                    .name("idx_user_sessions_user_id")
                    .table(UserSessions::Table)
                    .to_owned(),
            )
            .await
            .ok();

        // Drop table
        manager
            .drop_table(Table::drop().table(UserSessions::Table).to_owned())
            .await
    }
}
