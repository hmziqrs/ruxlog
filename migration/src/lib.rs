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
mod m20250510_000010_alter_category_add_color_is_active;
mod m20250811_000011_create_newsletter_subscribers_table;
mod m20250812_000012_create_post_revisions_table;
mod m20250812_000012_create_user_sessions_table;
mod m20250812_000013_alter_user_add_twofa_fields;
mod m20250812_000013_create_scheduled_posts_table;
mod m20250812_000014_create_post_series_tables;
mod m20250813_000015_alter_asset_context_enum;
mod m20250813_000016_alter_post_comment_add_moderation;
mod m20250813_000017_create_comment_flags_table;
mod m20250814_000018_create_media_table;
mod m20251029_000019_drop_asset_table;
mod m20251030_000021_alter_category_change_media_fields;
mod m20251116_000020_create_media_variant_table;
mod m20251116_000022_alter_media_add_hash;
mod m20251117_000023_create_media_usages_table;
mod m20251118_000024_alter_user_add_avatar_id;

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
            Box::new(m20250510_000010_alter_category_add_color_is_active::Migration),
            Box::new(m20250811_000011_create_newsletter_subscribers_table::Migration),
            Box::new(m20250812_000012_create_post_revisions_table::Migration),
            Box::new(m20250812_000013_create_scheduled_posts_table::Migration),
            Box::new(m20250812_000014_create_post_series_tables::Migration),
            Box::new(m20250812_000012_create_user_sessions_table::Migration),
            Box::new(m20250812_000013_alter_user_add_twofa_fields::Migration),
            Box::new(m20250813_000015_alter_asset_context_enum::Migration),
            Box::new(m20250813_000016_alter_post_comment_add_moderation::Migration),
            Box::new(m20250813_000017_create_comment_flags_table::Migration),
            Box::new(m20250814_000018_create_media_table::Migration),
            Box::new(m20251029_000019_drop_asset_table::Migration),
            Box::new(m20251116_000020_create_media_variant_table::Migration),
            Box::new(m20251116_000022_alter_media_add_hash::Migration),
            Box::new(m20251030_000021_alter_category_change_media_fields::Migration),
            Box::new(m20251117_000023_create_media_usages_table::Migration),
            Box::new(m20251118_000024_alter_user_add_avatar_id::Migration),
        ]
    }
}
