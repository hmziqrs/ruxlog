use sea_orm::{entity::prelude::*, ConnectionTrait, DbBackend, Order, QueryOrder, Set, Statement};
use crate::error::{DbResult, ErrorCode, ErrorResponse};

use super::*;

impl Entity {
    const PER_PAGE: u64 = 10;

    // Create a new post
    pub async fn create(conn: &DbConn, new_post: NewPost) -> DbResult<Model> {
        let now = chrono::Utc::now().naive_utc();
        let post = ActiveModel {
            title: Set(new_post.title),
            slug: Set(new_post.slug),
            content: Set(new_post.content),
            excerpt: Set(new_post.excerpt),
            featured_image: Set(new_post.featured_image),
            status: Set(new_post.status),
            published_at: Set(new_post.published_at),
            user_id: Set(new_post.user_id),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        match post.insert(conn).await {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    // Update an existing post
    pub async fn update(
        conn: &DbConn,
        post_id: i32,
        update_post: UpdatePost,
    ) -> DbResult<Option<Model>> {
        let post: Option<Model> = Self::find_by_id(post_id).one(conn).await?;

        if let Some(post_model) = post {
            let mut post_active: ActiveModel = post_model.into();

            if let Some(title) = update_post.title {
                post_active.title = Set(title);
            }

            if let Some(slug) = update_post.slug {
                post_active.slug = Set(slug);
            }

            if let Some(content) = update_post.content {
                post_active.content = Set(content);
            }

            if let Some(excerpt) = update_post.excerpt {
                post_active.excerpt = Set(Some(excerpt));
            }

            if let Some(featured_image) = update_post.featured_image {
                post_active.featured_image = Set(Some(featured_image));
            }

            if let Some(status) = update_post.status {
                post_active.status = Set(status);
            }

            if let Some(published_at) = update_post.published_at {
                post_active.published_at = Set(Some(published_at));
            }

            post_active.updated_at = Set(update_post.updated_at);

            match post_active.update(conn).await {
                Ok(updated_post) => Ok(Some(updated_post)),
                Err(err) => Err(err.into()),
            }
        } else {
            Ok(None)
        }
    }

    // Delete post
    pub async fn delete(conn: &DbConn, post_id: i32) -> DbResult<u64> {
        match Self::delete_by_id(post_id).exec(conn).await {
            Ok(result) => Ok(result.rows_affected),
            Err(err) => Err(err.into()),
        }
    }

    // Find post by ID
    pub async fn find_by_id_with_404(conn: &DbConn, post_id: i32) -> DbResult<Model> {
        match Self::find_by_id(post_id).one(conn).await {
            Ok(Some(model)) => Ok(model),
            Ok(None) => Err(ErrorResponse::new(ErrorCode::RecordNotFound)
                        .with_message(&format!("Post with ID {} not found", post_id))),
            Err(err) => Err(err.into()),
        }
    }

    // Find post by slug
    pub async fn find_by_slug(conn: &DbConn, post_slug: String) -> DbResult<Option<Model>> {
        match Self::find()
            .filter(Column::Slug.eq(post_slug))
            .one(conn)
            .await
        {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    // Search posts with query parameters
    pub async fn search(
        conn: &DbConn,
        query: PostQuery,
    ) -> DbResult<(Vec<Model>, u64)> {
        let mut post_query = Self::find();

        // Apply filters
        if let Some(title_filter) = query.title {
            let title_pattern = format!("%{}%", title_filter);
            post_query = post_query.filter(Column::Title.contains(&title_pattern));
        }

        if let Some(status_filter) = query.status {
            post_query = post_query.filter(Column::Status.eq(status_filter));
        }

        if let Some(user_id_filter) = query.user_id {
            post_query = post_query.filter(Column::UserId.eq(user_id_filter));
        }

        if let Some(created_at_filter) = query.created_at {
            post_query = post_query.filter(Column::CreatedAt.eq(created_at_filter));
        }

        if let Some(updated_at_filter) = query.updated_at {
            post_query = post_query.filter(Column::UpdatedAt.eq(updated_at_filter));
        }

        if let Some(published_at_filter) = query.published_at {
            post_query = post_query.filter(Column::PublishedAt.eq(published_at_filter));
        }

        // Handle sort_by fields
        if let Some(sort_fields) = &query.sort_by {
            for field in sort_fields {
                let order = if query.sort_order.as_deref() == Some("asc") {
                    Order::Asc
                } else {
                    Order::Desc
                };
                
                post_query = match field.as_str() {
                    "title" => post_query.order_by(Column::Title, order),
                    "status" => post_query.order_by(Column::Status, order),
                    "created_at" => post_query.order_by(Column::CreatedAt, order),
                    "updated_at" => post_query.order_by(Column::UpdatedAt, order),
                    "published_at" => post_query.order_by(Column::PublishedAt, order),
                    _ => post_query,
                };
            }
        } else {
            // Default ordering
            let order = if query.sort_order.as_deref() == Some("asc") {
                Order::Asc
            } else {
                Order::Desc
            };
            post_query = post_query.order_by(Column::CreatedAt, order);
        }

        // Handle pagination
        let page = match query.page_no {
            Some(p) if p > 0 => p,
            _ => 1,
        };
        
        let paginator = post_query.paginate(conn, Self::PER_PAGE);
        
        // Get total count and paginated results
        match paginator.num_items().await {
            Ok(total) => match paginator.fetch_page(page - 1).await {
                Ok(results) => Ok((results, total)),
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        }
    }

    // Get posts with user information
    pub async fn get_posts_with_user(
        conn: &DbConn,
        query: PostQuery,
    ) -> DbResult<(Vec<PostWithUser>, u64)> {
        // Custom SQL query to join posts with users
        let sql = r#"
            SELECT 
                p.id, p.title, p.slug, p.content, p.excerpt, p.featured_image, 
                p.status, p.published_at, p.created_at, p.updated_at, p.user_id,
                u.name as user_name, u.avatar as user_avatar
            FROM posts p
            JOIN users u ON p.user_id = u.id
            WHERE 1=1
        "#;

        // Build the WHERE clause based on the query parameters
        let mut where_clauses = Vec::new();
        let mut params = Vec::<Value>::new();

        if let Some(title) = &query.title {
            where_clauses.push("p.title LIKE ?");
            params.push(format!("%{}%", title).into());
        }

        if let Some(status) = &query.status {
            where_clauses.push("p.status = ?");
            params.push(status.to_string().into());
        }

        if let Some(user_id) = query.user_id {
            where_clauses.push("p.user_id = ?");
            params.push(user_id.into());
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
                "title" => "p.title",
                "created_at" => "p.created_at",
                "updated_at" => "p.updated_at",
                "published_at" => "p.published_at",
                _ => "p.created_at",
            },
            _ => "p.created_at",
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
        
        let posts: Vec<PostWithUser> = match conn.query_all(stmt).await {
            Ok(rows) => {
                rows.into_iter()
                    .map(|row| {
                        PostWithUser {
                            id: row.try_get("", "id").unwrap_or_default(),
                            title: row.try_get("", "title").unwrap_or_default(),
                            slug: row.try_get("", "slug").unwrap_or_default(),
                            content: row.try_get("", "content").unwrap_or_default(),
                            excerpt: row.try_get("", "excerpt").ok(),
                            featured_image: row.try_get("", "featured_image").ok(),
                            status: match row.try_get::<String>("", "status") {
                                Ok(s) if s == "published" => PostStatus::Published,
                                Ok(s) if s == "archived" => PostStatus::Archived,
                                _ => PostStatus::Draft,
                            },
                            published_at: row.try_get("", "published_at").ok(),
                            created_at: row.try_get("", "created_at").unwrap_or_default(),
                            updated_at: row.try_get("", "updated_at").unwrap_or_default(),
                            user_id: row.try_get("", "user_id").unwrap_or_default(),
                            user_name: row.try_get("", "user_name").unwrap_or_default(),
                            user_avatar: row.try_get("", "user_avatar").ok(),
                        }
                    })
                    .collect()
            },
            Err(err) => return Err(err.into()),
        };

        // Count total posts with the same filters but without pagination
        let mut count_sql = format!(
            "SELECT COUNT(*) as total FROM posts p JOIN users u ON p.user_id = u.id WHERE 1=1"
        );
        
        if !where_clauses.is_empty() {
            count_sql += &format!(" AND {}", where_clauses.join(" AND "));
        }

        let stmt = Statement::from_sql_and_values(DbBackend::Postgres, &count_sql, params);
        
        let count: u64 = match conn.query_one(stmt).await {
            Ok(Some(row)) => row.try_get("", "total").unwrap_or(0),
            _ => 0,
        };

        Ok((posts, count))
    }

    // Get posts with user information and stats (view and comment counts)
    pub async fn get_posts_with_stats(
        conn: &DbConn,
        query: PostQuery,
    ) -> DbResult<(Vec<PostWithStats>, u64)> {
        // Custom SQL query to join posts with users and count views and comments
        let sql = r#"
            SELECT 
                p.id, p.title, p.slug, p.content, p.excerpt, p.featured_image, 
                p.status, p.published_at, p.created_at, p.updated_at, p.user_id,
                u.name as user_name, u.avatar as user_avatar,
                COUNT(DISTINCT pv.id) as view_count,
                COUNT(DISTINCT pc.id) as comment_count
            FROM posts p
            JOIN users u ON p.user_id = u.id
            LEFT JOIN post_views pv ON p.id = pv.post_id
            LEFT JOIN post_comments pc ON p.id = pc.post_id
            WHERE 1=1
        "#;

        // Build the WHERE clause based on the query parameters
        let mut where_clauses = Vec::new();
        let mut params = Vec::<Value>::new();

        if let Some(title) = &query.title {
            where_clauses.push("p.title LIKE ?");
            params.push(format!("%{}%", title).into());
        }

        if let Some(status) = &query.status {
            where_clauses.push("p.status = ?");
            params.push(status.to_string().into());
        }

        if let Some(user_id) = query.user_id {
            where_clauses.push("p.user_id = ?");
            params.push(user_id.into());
        }

        // Build the final SQL query with group by
        let mut final_sql = sql.to_owned();
        
        // Add where clauses if any
        if !where_clauses.is_empty() {
            final_sql += &format!(" AND {}", where_clauses.join(" AND "));
        }

        // Add group by
        final_sql += " GROUP BY p.id, u.name, u.avatar";

        // Add order by clause
        let sort_field = match &query.sort_by {
            Some(fields) if !fields.is_empty() => match fields[0].as_str() {
                "title" => "p.title",
                "created_at" => "p.created_at",
                "updated_at" => "p.updated_at",
                "published_at" => "p.published_at",
                "view_count" => "view_count",
                "comment_count" => "comment_count",
                _ => "p.created_at",
            },
            _ => "p.created_at",
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
        
        let posts: Vec<PostWithStats> = match conn.query_all(stmt).await {
            Ok(rows) => {
                rows.into_iter()
                    .map(|row| {
                        PostWithStats {
                            id: row.try_get("", "id").unwrap_or_default(),
                            title: row.try_get("", "title").unwrap_or_default(),
                            slug: row.try_get("", "slug").unwrap_or_default(),
                            content: row.try_get("", "content").unwrap_or_default(),
                            excerpt: row.try_get("", "excerpt").ok(),
                            featured_image: row.try_get("", "featured_image").ok(),
                            status: match row.try_get::<String>("", "status") {
                                Ok(s) if s == "published" => PostStatus::Published,
                                Ok(s) if s == "archived" => PostStatus::Archived,
                                _ => PostStatus::Draft,
                            },
                            published_at: row.try_get("", "published_at").ok(),
                            created_at: row.try_get("", "created_at").unwrap_or_default(),
                            updated_at: row.try_get("", "updated_at").unwrap_or_default(),
                            user_id: row.try_get("", "user_id").unwrap_or_default(),
                            user_name: row.try_get("", "user_name").unwrap_or_default(),
                            user_avatar: row.try_get("", "user_avatar").ok(),
                            view_count: row.try_get("", "view_count").unwrap_or_default(),
                            comment_count: row.try_get("", "comment_count").unwrap_or_default(),
                        }
                    })
                    .collect()
            },
            Err(err) => return Err(err.into()),
        };

        // Count total posts with the same filters but without pagination
        let mut count_sql = format!(
            "SELECT COUNT(DISTINCT p.id) as total FROM posts p JOIN users u ON p.user_id = u.id WHERE 1=1"
        );
        
        if !where_clauses.is_empty() {
            count_sql += &format!(" AND {}", where_clauses.join(" AND "));
        }

        let stmt = Statement::from_sql_and_values(DbBackend::Postgres, &count_sql, params);
        
        let count: u64 = match conn.query_one(stmt).await {
            Ok(Some(row)) => row.try_get("", "total").unwrap_or(0),
            _ => 0,
        };

        Ok((posts, count))
    }
}