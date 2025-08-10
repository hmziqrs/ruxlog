use sea_orm::{
    entity::prelude::*, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, Set, TransactionTrait,
};

use crate::error::DbResult;

use super::*;

use super::model::ScheduledPostStatus;

/// Actions for scheduled posts:
/// - Create a schedule
/// - Upsert (create or update) a schedule for a post
/// - Query helpers (find by post, list pending due items, list by status)
impl Entity {
    pub const PER_PAGE: u64 = 10;

    /// Create a scheduled post entry.
    /// Defaults status to Pending if not provided.
    pub async fn create(
        conn: &DbConn,
        post_id: i32,
        publish_at: DateTimeWithTimeZone,
        status: Option<ScheduledPostStatus>,
    ) -> DbResult<Model> {
        let now = chrono::Utc::now().fixed_offset();

        let active = ActiveModel {
            post_id: Set(post_id),
            publish_at: Set(publish_at),
            status: Set(status.unwrap_or(ScheduledPostStatus::Pending)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        let model = active.insert(conn).await?;
        Ok(model)
    }

    /// Upsert a scheduled post for a given post_id.
    /// - If an existing Pending schedule exists for the post, update its publish_at and updated_at.
    /// - Otherwise, create a new Pending schedule.
    pub async fn upsert(
        conn: &DbConn,
        post_id: i32,
        publish_at: DateTimeWithTimeZone,
    ) -> DbResult<Model> {
        let now = chrono::Utc::now().fixed_offset();
        let txn = conn.begin().await?;

        // Find an existing pending schedule for this post
        let existing = Entity::find()
            .filter(Column::PostId.eq(post_id))
            .filter(Column::Status.eq(ScheduledPostStatus::Pending))
            .order_by_desc(Column::UpdatedAt)
            .order_by_desc(Column::Id)
            .one(&txn)
            .await?;

        let result = if let Some(existing_model) = existing {
            let mut active: ActiveModel = existing_model.into();
            active.publish_at = Set(publish_at);
            active.status = Set(ScheduledPostStatus::Pending);
            active.updated_at = Set(now);
            active.update(&txn).await?
        } else {
            let active = ActiveModel {
                post_id: Set(post_id),
                publish_at: Set(publish_at),
                status: Set(ScheduledPostStatus::Pending),
                created_at: Set(now),
                updated_at: Set(now),
                ..Default::default()
            };
            active.insert(&txn).await?
        };

        txn.commit().await?;
        Ok(result)
    }

    /// Find the latest schedule (by updated_at desc then id desc) for a given post.
    pub async fn find_by_post_id(conn: &DbConn, post_id: i32) -> DbResult<Option<Model>> {
        let model = Entity::find()
            .filter(Column::PostId.eq(post_id))
            .order_by_desc(Column::UpdatedAt)
            .order_by_desc(Column::Id)
            .one(conn)
            .await?;
        Ok(model)
    }

    /// Return pending scheduled posts due at or before the given timestamp.
    /// Results are ordered by publish_at asc, then id asc. Optional limit.
    pub async fn due_pending(
        conn: &DbConn,
        until: DateTimeWithTimeZone,
        limit: Option<u64>,
    ) -> DbResult<Vec<Model>> {
        let mut query = Entity::find()
            .filter(Column::Status.eq(ScheduledPostStatus::Pending))
            .filter(Column::PublishAt.lte(until))
            .order_by_asc(Column::PublishAt)
            .order_by_asc(Column::Id);

        if let Some(lim) = limit {
            query = query.limit(lim);
        }

        let items = query.all(conn).await?;
        Ok(items)
    }

    /// List scheduled posts by status with pagination (page starts at 1).
    /// Returns (items, total_count).
    pub async fn list_by_status(
        conn: &DbConn,
        status: ScheduledPostStatus,
        page: Option<u64>,
        per_page: Option<u64>,
    ) -> DbResult<(Vec<Model>, u64)> {
        let per_page = per_page.unwrap_or(Self::PER_PAGE);
        let page = match page {
            Some(p) if p > 0 => p,
            _ => 1,
        };

        let query = Entity::find()
            .filter(Column::Status.eq(status))
            .order_by_desc(Column::UpdatedAt)
            .order_by_desc(Column::Id);

        let paginator = query.paginate(conn, per_page);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(page - 1).await?;

        Ok((items, total))
    }
}
