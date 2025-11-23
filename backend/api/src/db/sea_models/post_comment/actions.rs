use crate::error::DbResult;
use sea_orm::{entity::prelude::*, Order, QueryOrder, Set};
use tracing::{error, info, instrument, warn};

use super::*;

impl Entity {
    pub const PER_PAGE: u64 = 20;

    #[instrument(skip(conn, new_comment), fields(comment_id, post_id = new_comment.post_id, user_id = new_comment.user_id))]
    pub async fn create(conn: &DbConn, new_comment: NewComment) -> DbResult<Model> {
        let now = chrono::Utc::now().fixed_offset();
        let comment = ActiveModel {
            post_id: Set(new_comment.post_id),
            user_id: Set(new_comment.user_id),
            content: Set(new_comment.content),
            likes_count: Set(new_comment.likes_count.unwrap_or(0)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        match comment.insert(conn).await {
            Ok(model) => {
                tracing::Span::current().record("comment_id", model.id);
                info!(
                    comment_id = model.id,
                    post_id = model.post_id,
                    user_id = model.user_id,
                    "Comment created"
                );
                Ok(model)
            }
            Err(err) => {
                error!(
                    post_id = new_comment.post_id,
                    user_id = new_comment.user_id,
                    "Failed to create comment: {}",
                    err
                );
                Err(err.into())
            }
        }
    }

    #[instrument(skip(conn, update_comment), fields(comment_id, user_id))]
    pub async fn update(
        conn: &DbConn,
        comment_id: i32,
        user_id: i32,
        update_comment: UpdateComment,
    ) -> DbResult<Option<Model>> {
        let comment: Option<Model> = Self::find_by_id(comment_id)
            .filter(Column::UserId.eq(user_id))
            .one(conn)
            .await?;

        if let Some(comment_model) = comment {
            let mut comment_active: ActiveModel = comment_model.into();

            if let Some(content) = update_comment.content {
                comment_active.content = Set(content);
            }

            comment_active.updated_at = Set(update_comment.updated_at);

            match comment_active.update(conn).await {
                Ok(updated_comment) => {
                    info!(comment_id, user_id, "Comment updated");
                    Ok(Some(updated_comment))
                }
                Err(err) => {
                    error!(comment_id, user_id, "Failed to update comment: {}", err);
                    Err(err.into())
                }
            }
        } else {
            warn!(comment_id, user_id, "Comment not found for update");
            Ok(None)
        }
    }

    #[instrument(skip(conn), fields(comment_id, user_id))]
    pub async fn delete(conn: &DbConn, comment_id: i32, user_id: i32) -> DbResult<u64> {
        match Self::delete_by_id(comment_id)
            .filter(Column::UserId.eq(user_id))
            .exec(conn)
            .await
        {
            Ok(result) => {
                info!(
                    comment_id,
                    user_id,
                    rows_affected = result.rows_affected,
                    "Comment deleted"
                );
                Ok(result.rows_affected)
            }
            Err(err) => {
                error!(comment_id, user_id, "Failed to delete comment: {}", err);
                Err(err.into())
            }
        }
    }

    /// Find all comments by post ID (public use)
    #[instrument(skip(conn), fields(post_id))]
    pub async fn find_all_by_post(conn: &DbConn, post_id: i32) -> DbResult<Vec<CommentWithUser>> {
        use super::super::user::Column as UserColumn;
        use sea_orm::prelude::Expr;
        use sea_orm::sea_query::Alias;
        use sea_orm::{JoinType, QuerySelect};

        let comments_joined = Self::find()
            .select_only()
            .column(Column::Id)
            .column(Column::PostId)
            .column(Column::UserId)
            .column(Column::Content)
            .column(Column::LikesCount)
            .column(Column::Hidden)
            .column(Column::FlagsCount)
            .column(Column::CreatedAt)
            .column(Column::UpdatedAt)
            .column_as(UserColumn::Name, "user_name")
            .column_as(UserColumn::AvatarId, "user_avatar_id")
            .join(JoinType::InnerJoin, Relation::User.def())
            .join_as(
                JoinType::LeftJoin,
                super::super::user::Relation::Media.def(),
                Alias::new("user_avatar_media"),
            )
            .expr_as(
                Expr::col((
                    Alias::new("user_avatar_media"),
                    super::super::media::Column::ObjectKey,
                )),
                "user_avatar_object_key",
            )
            .expr_as(
                Expr::col((
                    Alias::new("user_avatar_media"),
                    super::super::media::Column::FileUrl,
                )),
                "user_avatar_file_url",
            )
            .expr_as(
                Expr::col((
                    Alias::new("user_avatar_media"),
                    super::super::media::Column::MimeType,
                )),
                "user_avatar_mime_type",
            )
            .expr_as(
                Expr::col((
                    Alias::new("user_avatar_media"),
                    super::super::media::Column::Width,
                )),
                "user_avatar_width",
            )
            .expr_as(
                Expr::col((
                    Alias::new("user_avatar_media"),
                    super::super::media::Column::Height,
                )),
                "user_avatar_height",
            )
            .expr_as(
                Expr::col((
                    Alias::new("user_avatar_media"),
                    super::super::media::Column::Size,
                )),
                "user_avatar_size",
            )
            .filter(Column::PostId.eq(post_id))
            .filter(Column::Hidden.eq(false))
            .order_by(Column::CreatedAt, Order::Asc)
            .into_model::<CommentWithUserJoined>()
            .all(conn)
            .await?;

        let comments = comments_joined
            .into_iter()
            .map(|c| c.into_comment_with_user())
            .collect();

        Ok(comments)
    }

    /// Find comments with query (dashboard use)
    pub async fn find_with_query(
        conn: &DbConn,
        query: CommentQuery,
    ) -> DbResult<(Vec<CommentWithUser>, u64)> {
        use super::super::user::Column as UserColumn;
        use sea_orm::prelude::Expr;
        use sea_orm::sea_query::Alias;
        use sea_orm::{JoinType, QuerySelect};

        let mut comment_query = Self::find()
            .select_only()
            .column(Column::Id)
            .column(Column::PostId)
            .column(Column::UserId)
            .column(Column::Content)
            .column(Column::LikesCount)
            .column(Column::Hidden)
            .column(Column::FlagsCount)
            .column(Column::CreatedAt)
            .column(Column::UpdatedAt)
            .column_as(UserColumn::Name, "user_name")
            .column_as(UserColumn::AvatarId, "user_avatar_id")
            .join(JoinType::InnerJoin, Relation::User.def())
            .join_as(
                JoinType::LeftJoin,
                super::super::user::Relation::Media.def(),
                Alias::new("user_avatar_media"),
            )
            .expr_as(
                Expr::col((
                    Alias::new("user_avatar_media"),
                    super::super::media::Column::ObjectKey,
                )),
                "user_avatar_object_key",
            )
            .expr_as(
                Expr::col((
                    Alias::new("user_avatar_media"),
                    super::super::media::Column::FileUrl,
                )),
                "user_avatar_file_url",
            )
            .expr_as(
                Expr::col((
                    Alias::new("user_avatar_media"),
                    super::super::media::Column::MimeType,
                )),
                "user_avatar_mime_type",
            )
            .expr_as(
                Expr::col((
                    Alias::new("user_avatar_media"),
                    super::super::media::Column::Width,
                )),
                "user_avatar_width",
            )
            .expr_as(
                Expr::col((
                    Alias::new("user_avatar_media"),
                    super::super::media::Column::Height,
                )),
                "user_avatar_height",
            )
            .expr_as(
                Expr::col((
                    Alias::new("user_avatar_media"),
                    super::super::media::Column::Size,
                )),
                "user_avatar_size",
            );

        if let Some(post_id_filter) = query.post_id {
            comment_query = comment_query.filter(Column::PostId.eq(post_id_filter));
        }

        if let Some(user_id_filter) = query.user_id {
            comment_query = comment_query.filter(Column::UserId.eq(user_id_filter));
        }

        if let Some(search_term) = &query.search_term {
            comment_query = comment_query.filter(Column::Content.contains(search_term));
        }

        match query.hidden_filter.unwrap_or(HiddenFilter::Visible) {
            HiddenFilter::All => {}
            HiddenFilter::Hidden => {
                comment_query = comment_query.filter(Column::Hidden.eq(true));
            }
            HiddenFilter::Visible => {
                comment_query = comment_query.filter(Column::Hidden.eq(false));
            }
        }

        if let Some(min_flags) = query.min_flags {
            comment_query = comment_query.filter(Column::FlagsCount.gte(min_flags));
        }

        // Date range filters
        if let Some(ts) = query.created_at_gt {
            comment_query = comment_query.filter(Column::CreatedAt.gt(ts));
        }
        if let Some(ts) = query.created_at_lt {
            comment_query = comment_query.filter(Column::CreatedAt.lt(ts));
        }
        if let Some(ts) = query.updated_at_gt {
            comment_query = comment_query.filter(Column::UpdatedAt.gt(ts));
        }
        if let Some(ts) = query.updated_at_lt {
            comment_query = comment_query.filter(Column::UpdatedAt.lt(ts));
        }

        // Multi-field sorting with per-field order
        if let Some(sorts) = query.sorts {
            for sort in sorts {
                let column = match sort.field.as_str() {
                    "created_at" => Some(Column::CreatedAt),
                    "updated_at" => Some(Column::UpdatedAt),
                    "likes_count" => Some(Column::LikesCount),
                    "flags_count" => Some(Column::FlagsCount),
                    _ => None,
                };
                if let Some(col) = column {
                    comment_query = comment_query.order_by(col, sort.order);
                }
            }
        } else {
            comment_query = comment_query.order_by(Column::CreatedAt, Order::Desc);
        }

        let page = match query.page_no {
            Some(p) if p > 0 => p,
            _ => 1,
        };

        let paginator = comment_query
            .into_model::<CommentWithUserJoined>()
            .paginate(conn, Self::PER_PAGE);

        let total = paginator.num_items().await?;
        let models_joined = paginator.fetch_page(page - 1).await?;

        let models = models_joined
            .into_iter()
            .map(|c| c.into_comment_with_user())
            .collect();

        Ok((models, total))
    }

    pub async fn count_by_post_id(conn: &DbConn, post_id: i32) -> DbResult<i64> {
        let count = Self::find()
            .filter(Column::PostId.eq(post_id))
            .count(conn)
            .await?;

        Ok(count as i64)
    }

    pub async fn admin_hide(conn: &DbConn, comment_id: i32) -> DbResult<Option<Model>> {
        let existing = Self::find_by_id(comment_id).one(conn).await?;
        if let Some(model) = existing {
            let mut active: ActiveModel = model.into();
            active.hidden = Set(true);
            active.updated_at = Set(chrono::Utc::now().fixed_offset());
            let updated = active.update(conn).await?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }

    pub async fn admin_unhide(conn: &DbConn, comment_id: i32) -> DbResult<Option<Model>> {
        let existing = Self::find_by_id(comment_id).one(conn).await?;
        if let Some(model) = existing {
            let mut active: ActiveModel = model.into();
            active.hidden = Set(false);
            active.updated_at = Set(chrono::Utc::now().fixed_offset());
            let updated = active.update(conn).await?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }

    pub async fn admin_delete(conn: &DbConn, comment_id: i32) -> DbResult<u64> {
        let res = Self::delete_by_id(comment_id).exec(conn).await?;
        Ok(res.rows_affected)
    }

    pub async fn admin_flags_clear(conn: &DbConn, comment_id: i32) -> DbResult<Option<Model>> {
        let existing = Self::find_by_id(comment_id).one(conn).await?;
        if let Some(model) = existing {
            let mut active: ActiveModel = model.into();
            active.flags_count = Set(0);
            active.updated_at = Set(chrono::Utc::now().fixed_offset());
            let updated = active.update(conn).await?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }
}
