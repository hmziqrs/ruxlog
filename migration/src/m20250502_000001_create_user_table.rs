use sea_orm_migration::prelude::*;
// Import the user model from your main project
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"CREATE TYPE "user_role" AS ENUM ('super-admin', 'admin', 'moderator', 'author', 'user');"#
            )
            .await?;

        // Create users table
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Users::Name).string().not_null())
                    .col(
                        ColumnDef::new(Users::Email)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Users::Password).string().not_null())
                    .col(ColumnDef::new(Users::Avatar).string())
                    .col(
                        ColumnDef::new(Users::IsVerified)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(Users::Role)
                            .custom(UserRole::UserRole) // Use the Iden for the custom type
                            .not_null()
                            .default(Expr::val("user")),

                    )
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(r#"DROP TYPE "user_role";"#)
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        Ok(())
    }
}

// Define an Iden for the custom user_role type
#[derive(Iden)]
enum UserRole {
    #[iden = "user_role"]
    UserRole,
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
    Name,
    Email,
    Password,
    Avatar,
    IsVerified,
    Role,
    CreatedAt,
    UpdatedAt,
}
