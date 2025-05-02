use crate::error::{DbResult};
use sea_orm::{entity::prelude::*,  Order, QueryOrder, Set, };

use super::*;

impl Entity {
    const PER_PAGE: u64 = 20;

    // Create a new comment
    pub async fn create(conn: &DbConn, new_comment: NewComment) -> DbResult<Model> {
        let now = chrono::Utc::now().fixed_offset();
        let comment = ActiveModel {
            post_id: Set(new_comment.post_id),
            user_id: Set(new_comment.user_id),
            // parent_id field temporarily removed
            content: Set(new_comment.content),
            likes_count: Set(new_comment.likes_count.unwrap_or(0)),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        match comment.insert(conn).await {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    // Update an existing comment with user validation
    pub async fn update(
        conn: &DbConn,
        comment_id: i32,
        user_id: i32,
        update_comment: UpdateComment,
    ) -> DbResult<Option<Model>> {
        // Find the comment and verify ownership
        let comment: Option<Model> = Self::find_by_id(comment_id)
            .filter(Column::UserId.eq(user_id))
            .one(conn)
            .await?;

        if let Some(comment_model) = comment {
            let mut comment_active: ActiveModel = comment_model.into();

            // Only update content if it's provided
            if let Some(content) = update_comment.content {
                comment_active.content = Set(content);
            }

            comment_active.updated_at = Set(update_comment.updated_at);

            match comment_active.update(conn).await {
                Ok(updated_comment) => Ok(Some(updated_comment)),
                Err(err) => Err(err.into()),
            }
        } else {
            Ok(None)
        }
    }

    // Delete comment with user validation
    pub async fn delete(conn: &DbConn, comment_id: i32, user_id: i32) -> DbResult<u64> {
        match Self::delete_by_id(comment_id)
            .filter(Column::UserId.eq(user_id))
            .exec(conn)
            .await
        {
            Ok(result) => Ok(result.rows_affected),
            Err(err) => Err(err.into()),
        }
    }

    // Unified method for fetching comments with various filtering options - always including user data
    pub async fn get_comments(
        conn: &DbConn,
        query: CommentQuery,
    ) -> DbResult<(Vec<CommentWithUser>, u64)> {
        use super::super::user::Column as UserColumn;
        use sea_orm::{JoinType, QuerySelect};

        println!("Query: {:?}", query);

        // Start with a base query that joins comments with users
        let mut comment_query = Entity::find()
            .select_only()
            .column(Column::Id)
            .column(Column::PostId)
            .column(Column::UserId)
            // parent_id column temporarily removed
            .column(Column::Content)
            .column(Column::LikesCount)
            .column(Column::CreatedAt)
            .column(Column::UpdatedAt)
            .column_as(UserColumn::Name, "user_name")
            .column_as(UserColumn::Avatar, "user_avatar")
            .join(JoinType::InnerJoin, Relation::User.def());
        
        // Apply post_id filter if provided
        if let Some(post_id_filter) = query.post_id {
            comment_query = comment_query.filter(Column::PostId.eq(post_id_filter));
        }

        // Apply user_id filter if provided
        if let Some(user_id_filter) = query.user_id {
            comment_query = comment_query.filter(Column::UserId.eq(user_id_filter));
        }

        // Apply content search if provided
        if let Some(search_term) = &query.search_term {
            comment_query = comment_query.filter(Column::Content.contains(search_term));
        }

        // Handle sorting
        let order = if query.sort_order.as_deref() == Some("asc") {
            Order::Asc
        } else {
            Order::Desc
        };

        comment_query = match &query.sort_by {
            Some(fields) if !fields.is_empty() => match fields[0].as_str() {
                "created_at" => comment_query.order_by(Column::CreatedAt, order),
                "updated_at" => comment_query.order_by(Column::UpdatedAt, order),
                "likes_count" => comment_query.order_by(Column::LikesCount, order),
                _ => comment_query.order_by(Column::CreatedAt, order),
            },
            _ => comment_query.order_by(Column::CreatedAt, order),
        };

        // Handle pagination
        let page = match query.page_no {
            Some(p) if p > 0 => p,
            _ => 1,
        };

        // Create paginator and fetch results
        let paginator = comment_query
            .into_model::<CommentWithUser>()
            .paginate(conn, Self::PER_PAGE);

        // Get total count
        let total = paginator.num_items().await?;

        // Get paginated results
        let models = paginator.fetch_page(page - 1).await?;

        Ok((models, total))
    }

    // Count comments by post ID
    pub async fn count_by_post_id(conn: &DbConn, post_id: i32) -> DbResult<i64> {
        let count = Self::find()
            .filter(Column::PostId.eq(post_id))
            .count(conn)
            .await?;

        Ok(count as i64)
    }
}
