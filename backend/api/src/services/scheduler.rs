use std::time::Duration;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use tracing::{error, info, instrument};

use crate::db::sea_models::post::{ActiveModel, Column, Entity, PostStatus};
use crate::state::AppState;

/// Interval between scheduler ticks in seconds.
const TICK_INTERVAL_SECS: u64 = 60;

/// Start the scheduled post publisher as a background tokio task.
///
/// Every `TICK_INTERVAL_SECS` seconds this task queries for posts whose
/// `status` is `Draft` and `published_at` is set to a time in the past
/// (or exactly now).  Matching posts are transitioned to `Published`.
pub fn start_scheduler(state: AppState) {
    tokio::spawn(run(state));
    info!("Scheduled post publisher started (interval: {TICK_INTERVAL_SECS}s)");
}

async fn run(state: AppState) {
    let mut interval = tokio::time::interval(Duration::from_secs(TICK_INTERVAL_SECS));

    loop {
        interval.tick().await;
        if let Err(err) = publish_due_posts(&state).await {
            error!(error = %err, "Scheduled post publisher tick failed");
        }
    }
}

#[instrument(skip_all)]
async fn publish_due_posts(state: &AppState) -> Result<(), sea_orm::DbErr> {
    let now = chrono::Utc::now().fixed_offset();

    // Find draft posts whose scheduled publish time has arrived.
    let due_posts = Entity::find()
        .filter(Column::Status.eq(PostStatus::Draft))
        .filter(Column::PublishedAt.is_not_null())
        .filter(Column::PublishedAt.lte(now))
        .all(&state.sea_db)
        .await?;

    if due_posts.is_empty() {
        return Ok(());
    }

    let count = due_posts.len();
    for post in due_posts {
        let post_id = post.id;
        let mut active: ActiveModel = post.into();
        active.status = Set(PostStatus::Published);
        if let Err(err) = active.update(&state.sea_db).await {
            error!(
                post_id,
                error = %err,
                "Failed to publish scheduled post"
            );
        } else {
            info!(post_id, "Scheduled post published");
        }
    }

    info!(count, "Scheduled post publisher tick completed");
    Ok(())
}
