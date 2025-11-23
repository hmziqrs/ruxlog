use std::collections::HashMap;

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::db::sea_models::{
    category, comment_flag, email_verification, forgot_password, media, media_usage, media_variant,
    newsletter_subscriber, post, post_comment, post_revision, post_series, post_view, route_status,
    scheduled_post, seed_run, tag, user, user_session,
};

use super::types::{SeedError, SeedResult, TableRange};

#[derive(Debug, Clone)]
pub struct UndoOutcome {
    pub deleted: HashMap<String, u64>,
}

/// Undo a specific seed run based on ID ranges.
pub async fn undo_seed_run(db: &DatabaseConnection, run_id: i32) -> SeedResult<UndoOutcome> {
    let run = seed_run::Entity::find_by_id(run_id)
        .one(db)
        .await?
        .ok_or_else(|| SeedError::Db("Seed run not found".to_string()))?;

    let ranges: HashMap<String, TableRange> =
        serde_json::from_value(run.ranges).unwrap_or_default();

    let mut deleted: HashMap<String, u64> = HashMap::new();

    macro_rules! del_range {
        ($name:literal, $entity:path, $column:expr) => {
            if let Some(range) = ranges.get($name) {
                if range.to > 0 && range.to >= range.from {
                    let res = <$entity as sea_orm::EntityTrait>::delete_many()
                        .filter($column.gte(range.from))
                        .filter($column.lte(range.to))
                        .exec(db)
                        .await?;
                    deleted.insert($name.to_string(), res.rows_affected);
                }
            }
        };
    }

    // Dependency-aware order
    del_range!(
        "comment_flags",
        comment_flag::Entity,
        comment_flag::Column::Id
    );
    del_range!("post_views", post_view::Entity, post_view::Column::Id);
    del_range!(
        "post_comments",
        post_comment::Entity,
        post_comment::Column::Id
    );
    del_range!(
        "post_revisions",
        post_revision::Entity,
        post_revision::Column::Id
    );
    // post_series_posts not tracked; skip.
    del_range!("post_series", post_series::Entity, post_series::Column::Id);
    del_range!(
        "scheduled_posts",
        scheduled_post::Entity,
        scheduled_post::Column::Id
    );
    del_range!("media_usage", media_usage::Entity, media_usage::Column::Id);
    del_range!(
        "media_variants",
        media_variant::Entity,
        media_variant::Column::Id
    );
    del_range!("media", media::Entity, media::Column::Id);
    del_range!("posts", post::Entity, post::Column::Id);
    del_range!("tags", tag::Entity, tag::Column::Id);
    del_range!("categories", category::Entity, category::Column::Id);
    del_range!(
        "user_sessions",
        user_session::Entity,
        user_session::Column::Id
    );
    del_range!(
        "email_verifications",
        email_verification::Entity,
        email_verification::Column::Id
    );
    del_range!(
        "forgot_passwords",
        forgot_password::Entity,
        forgot_password::Column::Id
    );
    del_range!(
        "newsletter_subscribers",
        newsletter_subscriber::Entity,
        newsletter_subscriber::Column::Id
    );
    del_range!(
        "route_status",
        route_status::Entity,
        route_status::Column::Id
    );
    del_range!("users", user::Entity, user::Column::Id);

    Ok(UndoOutcome { deleted })
}
