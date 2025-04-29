use crate::error::{DbResult, ErrorCode, ErrorResponse};
use sea_orm::{entity::prelude::*, ConnectionTrait, DbBackend, Order, QueryOrder, Set, Statement};

use super::*;

impl Entity {
    const PER_PAGE: u64 = 20;

    // Create a new comment
    pub async fn create(conn: &DbConn, new_comment: NewComment) -> DbResult<Model> {
        let now = chrono::Utc::now().naive_utc();
        let comment = ActiveModel {
            post_id: Set(new_comment.post_id),
            user_id: Set(new_comment.user_id),
            parent_id: Set(new_comment.parent_id),
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

    // Find comment by ID with not found handling
    pub async fn find_by_id_with_404(conn: &DbConn, comment_id: i32) -> DbResult<Model> {
        match Self::find_by_id(comment_id).one(conn).await {
            Ok(Some(model)) => Ok(model),
            Ok(None) => Err(ErrorResponse::new(ErrorCode::RecordNotFound)
                .with_message(&format!("Comment with ID {} not found", comment_id))),
            Err(err) => Err(err.into()),
        }
    }

    // Find comments with query parameters
    pub async fn search(conn: &DbConn, query: CommentQuery) -> DbResult<(Vec<Model>, u64)> {
        let mut comment_query = Self::find().filter(Column::PostId.eq(query.post_id));

        // Apply filters
        if let Some(user_id) = query.user_id {
            comment_query = comment_query.filter(Column::UserId.eq(user_id));
        }

        if let Some(parent_id) = query.parent_id {
            comment_query = comment_query.filter(Column::ParentId.eq(parent_id));
        } else {
            // If parent_id is not provided, filter for root comments (where parent_id is null)
            comment_query = comment_query.filter(Column::ParentId.is_null());
        }

        // Apply content search if provided
        if let Some(search_term) = &query.search_term {
            comment_query = comment_query.filter(Column::Content.contains(search_term));
        }

        // Handle sort_by fields
        if let Some(sort_fields) = &query.sort_by {
            for field in sort_fields {
                let order = if query.sort_order.as_deref() == Some("asc") {
                    Order::Asc
                } else {
                    Order::Desc
                };

                comment_query = match field.as_str() {
                    "created_at" => comment_query.order_by(Column::CreatedAt, order),
                    "updated_at" => comment_query.order_by(Column::UpdatedAt, order),
                    "likes_count" => comment_query.order_by(Column::LikesCount, order),
                    _ => comment_query,
                };
            }
        } else {
            // Default ordering
            comment_query = comment_query.order_by(Column::CreatedAt, Order::Desc);
        }

        // Handle pagination
        let page = match query.page_no {
            Some(p) if p > 0 => p,
            _ => 1,
        };

        let paginator = comment_query.paginate(conn, Self::PER_PAGE);

        // Get total count and paginated results
        match paginator.num_items().await {
            Ok(total) => match paginator.fetch_page(page - 1).await {
                Ok(results) => Ok((results, total)),
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        }
    }

    // Get comments with user information using Sea ORM's query builder
    pub async fn get_comments_with_user(
        conn: &DbConn,
        query: CommentQuery,
    ) -> DbResult<(Vec<CommentWithUser>, u64)> {
        use super::super::user::Column as UserColumn;
        use sea_orm::{JoinType, QuerySelect};

        // Start with a base query that joins comments with users
        let mut comment_query = Entity::find()
            .select_only()
            .column(Column::Id)
            .column(Column::PostId)
            .column(Column::UserId)
            .column(Column::ParentId)
            .column(Column::Content)
            .column(Column::LikesCount)
            .column(Column::CreatedAt)
            .column(Column::UpdatedAt)
            .column_as(UserColumn::Name, "user_name")
            .column_as(UserColumn::Avatar, "user_avatar")
            .join(JoinType::InnerJoin, Relation::User.def())
            .filter(Column::PostId.eq(query.post_id));

        // Apply user filter if present
        if let Some(user_id) = query.user_id {
            comment_query = comment_query.filter(Column::UserId.eq(user_id));
        }

        // Apply parent filter
        if let Some(parent_id) = query.parent_id {
            comment_query = comment_query.filter(Column::ParentId.eq(parent_id));
        } else {
            comment_query = comment_query.filter(Column::ParentId.is_null());
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

        let paginator = comment_query.paginate(conn, Self::PER_PAGE);

        // Get total count
        let total = paginator.num_items().await?;

        // Get paginated results and convert them to CommentWithUser
        let models: Vec<Model> = paginator.fetch_page(page - 1).await?;

        // Convert the Model objects to CommentWithUser objects
        let comments: Vec<CommentWithUser> = models
            .into_iter()
            .map(|model| {
                // Extract user_name and user_avatar from model using appropriate methods
                // This will need proper implementation based on your actual Model structure
                CommentWithUser {
                    id: model.id,
                    post_id: model.post_id,
                    user_id: model.user_id,
                    parent_id: model.parent_id,
                    content: model.content,
                    likes_count: model.likes_count,
                    created_at: model.created_at,
                    updated_at: model.updated_at,
                    user_name: model.user_name,
                    user_avatar: model.user_avatar,
                }
            })
            .collect();

        Ok((comments, total))
    }

    // Get comment tree structure
    pub async fn get_comment_tree(conn: &DbConn, post_id: i32) -> DbResult<Vec<CommentTree>> {
        // Get all root comments
        let root_query = CommentQuery {
            page_no: Some(1),
            post_id,
            user_id: None,
            parent_id: None,
            search_term: None,
            sort_by: Some(vec!["created_at".to_string()]),
            sort_order: Some("desc".to_string()),
        };

        let (root_comments, _) = Self::get_comments_with_user(conn, root_query).await?;
        let mut tree = Vec::new();

        // For each root comment, fetch its replies
        for root_comment in root_comments {
            let replies_query = CommentQuery {
                page_no: Some(1),
                post_id,
                user_id: None,
                parent_id: Some(root_comment.id),
                search_term: None,
                sort_by: Some(vec!["created_at".to_string()]),
                sort_order: Some("asc".to_string()),
            };

            let (replies, _) = Self::get_comments_with_user(conn, replies_query).await?;

            tree.push(CommentTree {
                comment: root_comment,
                replies,
            });
        }

        Ok(tree)
    }

    // Count comments by post ID
    pub async fn count_by_post_id(conn: &DbConn, post_id: i32) -> DbResult<i64> {
        let count = Self::find()
            .filter(Column::PostId.eq(post_id))
            .count(conn)
            .await?;

        Ok(count as i64)
    }

    // List all comments (similar to previous implementation)
    pub async fn list_all(conn: &DbConn) -> DbResult<Vec<Model>> {
        let comments = Self::find().all(conn).await?;
        Ok(comments)
    }

    // List comments by user (similar to previous implementation)
    pub async fn list_by_user(
        conn: &DbConn,
        query_user_id: i32,
        page: u64,
    ) -> DbResult<(Vec<Model>, u64)> {
        let query = CommentQuery {
            page_no: Some(page),
            post_id: 0, // This is a placeholder as the search will override it
            user_id: Some(query_user_id),
            parent_id: None,
            search_term: None,
            sort_by: Some(vec!["created_at".to_string()]),
            sort_order: Some("desc".to_string()),
        };

        // Use a custom query just for user comments that ignores post_id
        let user_comments_query = Self::find()
            .filter(Column::UserId.eq(query_user_id))
            .order_by(Column::CreatedAt, Order::Desc);

        let paginator = user_comments_query.paginate(conn, Self::PER_PAGE);

        // Get total count and paginated results
        match paginator.num_items().await {
            Ok(total) => match paginator.fetch_page(page - 1).await {
                Ok(results) => Ok((results, total)),
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        }
    }

    // Find comments by content search (similar to previous implementation)
    pub async fn search_by_content(
        conn: &DbConn,
        search_term: &str,
        page: u64,
    ) -> DbResult<(Vec<Model>, u64)> {
        let content_search_query = Self::find()
            .filter(Column::Content.contains(search_term))
            .order_by(Column::CreatedAt, Order::Desc);

        let paginator = content_search_query.paginate(conn, Self::PER_PAGE);

        // Get total count and paginated results
        match paginator.num_items().await {
            Ok(total) => match paginator.fetch_page(page - 1).await {
                Ok(results) => Ok((results, total)),
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        }
    }
}
