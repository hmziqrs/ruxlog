pub use sea_orm_migration::prelude::*;

mod m20250502_000001_create_user_table;
mod m20250502_000002_create_email_verification_table;
mod m20250502_000003_create_forgot_password_table;
mod m20250502_000004_create_category_table;
mod m20250502_000005_create_tag_table;
mod m20250502_000006_create_post_table;
mod m20250502_000007_create_post_comment_table;
mod m20250502_000008_create_post_view_table;
mod m20250503_000001_create_asset_table;
mod m20250509_000009_alter_tag_add_appearance;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250502_000001_create_user_table::Migration),
            Box::new(m20250502_000002_create_email_verification_table::Migration),
            Box::new(m20250502_000003_create_forgot_password_table::Migration),
            Box::new(m20250502_000004_create_category_table::Migration),
            Box::new(m20250502_000005_create_tag_table::Migration),
            Box::new(m20250502_000006_create_post_table::Migration),
            Box::new(m20250502_000007_create_post_comment_table::Migration),
            Box::new(m20250502_000008_create_post_view_table::Migration),
            Box::new(m20250503_000001_create_asset_table::Migration),
            Box::new(m20250509_000009_alter_tag_add_appearance::Migration),
        ]
    }
}
