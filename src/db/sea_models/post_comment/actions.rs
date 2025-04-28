use sea_orm::{entity::prelude::*, ConnectionTrait, DbBackend, Order, QueryOrder, Set, Statement};
use crate::error::{DbResult, ErrorCode, ErrorResponse};

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
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        match comment.insert(conn).await {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    // Update an existing comment
    pub async fn update(
        conn: &DbConn,
        comment_id: i32,
        update_comment: UpdateComment,
    ) -> DbResult<Option<Model>> {
        let comment: Option<Model> = Self::find_by_id(comment_id).one(conn).await?;

        if let Some(comment_model) = comment {
            let mut comment_active: ActiveModel = comment_model.into();
            comment_active.content = Set(update_comment.content);
            comment_active.updated_at = Set(update_comment.updated_at);

            match comment_active.update(conn).await {
                Ok(updated_comment) => Ok(Some(updated_comment)),
                Err(err) => Err(err.into()),
            }
        } else {
            Ok(None)
        }
    }

    // Delete comment
    pub async fn delete(conn: &DbConn, comment_id: i32) -> DbResult<u64> {
        match Self::delete_by_id(comment_id).exec(conn).await {
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
    pub async fn search(
        conn: &DbConn,
        query: CommentQuery,
    ) -> DbResult<(Vec<Model>, u64)> {
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

    // Get comments with user information
    pub async fn get_comments_with_user(
        conn: &DbConn,
        query: CommentQuery,
    ) -> DbResult<(Vec<CommentWithUser>, u64)> {
        // Custom SQL query to join comments with users
        let sql = r#"
            SELECT 
                c.id, c.post_id, c.user_id, c.parent_id, c.content, 
                c.created_at, c.updated_at,
                u.name as user_name, u.avatar as user_avatar
            FROM post_comments c
            JOIN users u ON c.user_id = u.id
            WHERE c.post_id = ?
        "#;

        // Build the WHERE clause based on the query parameters
        let mut where_clauses = Vec::new();
        let mut params = Vec::<Value>::new();
        
        params.push(query.post_id.into());

        if let Some(user_id) = query.user_id {
            where_clauses.push("c.user_id = ?");
            params.push(user_id.into());
        }

        if let Some(parent_id) = query.parent_id {
            where_clauses.push("c.parent_id = ?");
            params.push(parent_id.into());
        } else {
            where_clauses.push("c.parent_id IS NULL");
        }

        // Build the final SQL query with pagination
        let mut final_sql = sql.to_owned();
        
        // Add where clauses if any
        if !where_clauses.is_empty() {
            final_sql += &format!(" AND {}", where_clauses.join(" AND "));
        }

        // Add order by clause
        let sort_field = match &query.sort_by {
            Some(fields) if !fields.is_empty() => match fields[0].as_str() {
                "created_at" => "c.created_at",
                "updated_at" => "c.updated_at",
                _ => "c.created_at",
            },
            _ => "c.created_at",
        };

        let sort_order = if query.sort_order.as_deref() == Some("asc") {
            "ASC"
        } else {
            "DESC"
        };

        final_sql += &format!(" ORDER BY {} {}", sort_field, sort_order);

        // Add pagination
        let page = match query.page_no {
            Some(p) if p > 0 => p,
            _ => 1,
        };
        
        let offset = (page - 1) * Self::PER_PAGE;
        final_sql += &format!(" LIMIT {} OFFSET {}", Self::PER_PAGE, offset);

        // Execute the query
        let stmt = Statement::from_sql_and_values(DbBackend::Postgres, &final_sql, params.clone());
        
        let comments: Vec<CommentWithUser> = match conn.query_all(stmt).await {
            Ok(rows) => {
                rows.into_iter()
                    .map(|row| {
                        CommentWithUser {
                            id: row.try_get("", "id").unwrap_or_default(),
                            post_id: row.try_get("", "post_id").unwrap_or_default(),
                            user_id: row.try_get("", "user_id").unwrap_or_default(),
                            parent_id: row.try_get("", "parent_id").ok(),
                            content: row.try_get("", "content").unwrap_or_default(),
                            created_at: row.try_get("", "created_at").unwrap_or_default(),
                            updated_at: row.try_get("", "updated_at").unwrap_or_default(),
                            user_name: row.try_get("", "user_name").unwrap_or_default(),
                            user_avatar: row.try_get("", "user_avatar").ok(),
                        }
                    })
                    .collect()
            },
            Err(err) => return Err(err.into()),
        };

        // Count total comments with the same filters but without pagination
        let mut count_sql = format!(
            "SELECT COUNT(*) as total FROM post_comments c JOIN users u ON c.user_id = u.id WHERE c.post_id = ?"
        );
        
        if !where_clauses.is_empty() {
            count_sql += &format!(" AND {}", where_clauses.join(" AND "));
        }

        let stmt = Statement::from_sql_and_values(DbBackend::Postgres, &count_sql, params);
        
        let count: u64 = match conn.query_one(stmt).await {
            Ok(Some(row)) => row.try_get("", "total").unwrap_or(0),
            _ => 0,
        };

        Ok((comments, count))
    }

    // Get comment tree structure
    pub async fn get_comment_tree(
        conn: &DbConn,
        post_id: i32,
    ) -> DbResult<Vec<CommentTree>> {
        // Get all root comments
        let root_query = CommentQuery {
            page_no: Some(1),
            post_id,
            user_id: None,
            parent_id: None,
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
}