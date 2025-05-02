use sea_orm_migration::prelude::*;
// Import the user model from your main project
use ruxlog::db::sea_models::user;
use sea_orm::{DbBackend, Schema};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let schema = Schema::new(DbBackend::Postgres);

        // Create user_role enum type by directly using the enum values from your user model
        let user_role_values: Vec<String> = user::UserRole::iter()
            .map(|role| role.to_string())
            .collect();

        manager
            .create_type(
                schema.create_enum_from_entity::<user::UserRole>()
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
                    .col(ColumnDef::new(Users::Email).string().not_null().unique_key())
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
                            .enumeration("user_role", vec![
                                "super-admin",
                                "admin",
                                "moderator",
                                "author",
                                "user",
                            ])
                            .not_null()
                            .default("user"),
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
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;
        
        manager
            .drop_type(Type::drop().name("user_role").to_owned())
            .await
    }
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
