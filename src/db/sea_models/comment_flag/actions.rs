use crate::error::DbResult;
use sea_orm::{
    entity::prelude::*, ColumnTrait, EntityTrait, JoinType, Order, QueryFilter, QueryOrder,
    QuerySelect, Set,
};

use super::{slice::*, *};

impl Entity {
    const PER_PAGE: u64 = 20;

    /// Create or upsert a comment flag (unique per (comment_id, user_id)).
    /// After creating/updating, sync the `flags_count` on the related post_comment.
    pub async fn create(conn: &DbConn, new_flag: NewCommentFlag) -> DbResult<Model> {
        // Upsert-like behavior: if a flag by this user for this comment exists, update the reason.
        let existing = Entity::find()
            .filter(Column::CommentId.eq(new_flag.comment_id))
            .filter(Column::UserId.eq(new_flag.user_id))
            .one(conn)
            .await?;

        let now = chrono::Utc::now().fixed_offset();

        let model = if let Some(found) = existing {
            let mut active: ActiveModel = found.into();
            active.reason = Set(new_flag.reason.clone());
            // Keep created_at as is
            active.update(conn).await?
        } else {
            let active = ActiveModel {
                comment_id: Set(new_flag.comment_id),
                user_id: Set(new_flag.user_id),
                reason: Set(new_flag.reason.clone()),
                created_at: Set(now),
                ..Default::default()
            };
            active.insert(conn).await?
        };

        // Sync flags_count on post_comment
        let _ = Self::sync_flags_count(conn, new_flag.comment_id).await?;

        Ok(model)
    }

    /// List flags with pagination and optional filters, joined with user info.
    pub async fn list(
        conn: &DbConn,
        query: CommentFlagQuery,
    ) -> DbResult<(Vec<FlagWithUser>, u64)> {
        use super::super::user::Column as UserColumn;

        let mut q = Entity::find()
            .select_only()
            .column(Column::Id)
            .column(Column::CommentId)
            .column(Column::UserId)
            .column(Column::Reason)
            .column(Column::CreatedAt)
            .column_as(UserColumn::Name, "user_name")
            .column_as(UserColumn::AvatarId, "user_avatar")
            .join(JoinType::InnerJoin, Relation::User.def());

        if let Some(comment_id) = query.comment_id {
            q = q.filter(Column::CommentId.eq(comment_id));
        }
        if let Some(user_id) = query.user_id {
            q = q.filter(Column::UserId.eq(user_id));
        }
        if let Some(term) = &query.search_term {
            q = q.filter(Column::Reason.contains(term));
        }

        let order = if query.sort_order.as_deref() == Some("asc") {
            Order::Asc
        } else {
            Order::Desc
        };

        q = match &query.sort_by {
            Some(fields) if !fields.is_empty() => match fields[0].as_str() {
                "created_at" => q.order_by(Column::CreatedAt, order),
                _ => q.order_by(Column::CreatedAt, order),
            },
            _ => q.order_by(Column::CreatedAt, order),
        };

        let page = match query.page_no {
            Some(p) if p > 0 => p,
            _ => 1,
        };

        let paginator = q
            .into_model::<FlagWithUser>()
            .paginate(conn, Self::PER_PAGE);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(page - 1).await?;

        Ok((items, total))
    }

    /// Return a summary for a specific comment's flags.
    pub async fn summary_for_comment(conn: &DbConn, comment_id: i32) -> DbResult<FlagsSummary> {
        let count = Entity::find()
            .filter(Column::CommentId.eq(comment_id))
            .count(conn)
            .await?;

        Ok(FlagsSummary {
            comment_id,
            flags_count: count as i64,
        })
    }

    /// Recalculate and persist the flags_count on the related post_comment.
    /// Returns the updated count.
    pub async fn sync_flags_count(conn: &DbConn, comment_id: i32) -> DbResult<i64> {
        use super::super::post_comment::{
            Column as PostCommentColumn, Entity as PostCommentEntity,
        };

        let count = Entity::find()
            .filter(Column::CommentId.eq(comment_id))
            .count(conn)
            .await? as i64;

        if let Some(pc) = PostCommentEntity::find_by_id(comment_id).one(conn).await? {
            let mut active: super::super::post_comment::ActiveModel = pc.into();
            active.flags_count = Set(count as i32);
            active.updated_at = Set(chrono::Utc::now().fixed_offset());
            let _ = active.update(conn).await?;
        }
        Ok(count)
    }

    /// Clear all flags for a comment and sync flags_count to 0.
    /// Returns number of deleted rows.
    pub async fn clear_flags(conn: &DbConn, comment_id: i32) -> DbResult<u64> {
        let res = Entity::delete_many()
            .filter(Column::CommentId.eq(comment_id))
            .exec(conn)
            .await?;
        let _ = Self::sync_flags_count(conn, comment_id).await?;
        Ok(res.rows_affected)
    }
}
