use crate::{db::sea_models::tag, error::{DbResult, ErrorCode, ErrorResponse}};
use sea_orm::{
    entity::prelude::*, Condition, ConnectionTrait, DbBackend, Order, QueryOrder, Set, Statement,
    TransactionTrait,
};

use super::*;

impl Entity {
    pub const PER_PAGE: u64 = 10;

    async fn sanitized_tag_ids(
        conn: &DbConn,
        tag_ids: Vec<i32>,
    ) -> DbResult<Vec<i32>> {
        let mut sanitized_ids = Vec::new();
        tag::Entity::find()
            .filter(tag::Column::Id.is_in(tag_ids))
            .all(conn)
            .await?
            .iter()
            .for_each(|tag| {
                sanitized_ids.push(tag.id);
            });
        Ok(sanitized_ids)
    }

    // Create a new post
    pub async fn create(conn: &DbConn, new_post: NewPost) -> DbResult<Model> {
        let now = chrono::Utc::now().fixed_offset();

        let sanitized_tag_ids = Self::sanitized_tag_ids(conn, new_post.tag_ids).await?;
        
        let post = ActiveModel {
            title: Set(new_post.title),
            slug: Set(new_post.slug),
            content: Set(new_post.content),
            excerpt: Set(new_post.excerpt),
            featured_image: Set(new_post.featured_image),
            status: Set(new_post.status),
            published_at: Set(new_post.published_at),
            author_id: Set(new_post.author_id),
            category_id: Set(new_post.category_id),
            view_count: Set(new_post.view_count),
            likes_count: Set(new_post.likes_count),
            tag_ids: Set(sanitized_tag_ids),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        match post.insert(conn).await {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    // Update an existing post with role-based access control
    pub async fn update_with_auth(
        conn: &DbConn,
        post_id: i32,
        update_post: UpdatePost,
        user: &super::super::user::Model,
        is_mod: bool,
    ) -> DbResult<Option<Model>> {
        // Check permissions - only mods can edit any post, others can edit their own
        let mut query = Self::find_by_id(post_id);

        if !is_mod {
            query = query.filter(Column::AuthorId.eq(user.id));
        }

        let post = query.one(conn).await?;

        if let Some(_) = post {
            return Self::update(conn, post_id, update_post).await;
        }

        Ok(None)
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

            if let Some(category_id) = update_post.category_id {
                post_active.category_id = Set(category_id);
            }

            if let Some(view_count) = update_post.view_count {
                post_active.view_count = Set(view_count);
            }

            if let Some(likes_count) = update_post.likes_count {
                post_active.likes_count = Set(likes_count);
            }

            if let Some(tag_ids) = update_post.tag_ids {
                post_active.tag_ids = Set(tag_ids);
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

    // Delete post with role-based access control
    pub async fn delete_with_auth(
        conn: &DbConn,
        post_id: i32,
        user: &super::super::user::Model,
        is_mod: bool,
    ) -> DbResult<u64> {
        // Only mods can delete any post, others can delete their own
        if !is_mod {
            // Check if the post belongs to the user
            let post = Self::find_by_id(post_id)
                .filter(Column::AuthorId.eq(user.id))
                .one(conn)
                .await?;

            if post.is_none() {
                return Ok(0); // Post doesn't exist or user doesn't have permission
            }
        }

        Self::delete(conn, post_id).await
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

    // Find post by ID or slug with relations
    pub async fn find_by_id_or_slug(
        conn: &DbConn,
        post_id: Option<i32>,
        post_slug: Option<String>,
    ) -> DbResult<Option<PostWithRelations>> {
        // Find the post by either ID or slug
        let post = match (post_id, post_slug.clone()) {
            (Some(id), _) => Self::find_by_id(id).one(conn).await?,
            (_, Some(slug)) => Self::find_by_slug(conn, slug).await?,
            _ => return Ok(None),
        };

        if let Some(post) = post {
            // Get user info
            let user = super::super::user::Entity::find_by_id(post.author_id)
                .one(conn)
                .await?
                .ok_or_else(|| {
                    ErrorResponse::new(ErrorCode::RecordNotFound)
                        .with_message(&format!("User with ID {} not found", post.author_id))
                })?;

            // Get category if present
            let category = if let Some(cat_id) = post.category_id {
                match super::super::category::Entity::find_by_id(cat_id)
                    .one(conn)
                    .await?
                {
                    Some(cat) => Some(PostCategory {
                        id: cat.id,
                        name: cat.name.clone(),
                    }),
                    None => None,
                }
            } else {
                None
            };

            // Get tags
            let mut tags = Vec::new();
            if !post.tag_ids.is_empty() {
                let tag_models = super::super::tag::Entity::find()
                    .filter(super::super::tag::Column::Id.is_in(post.tag_ids.clone()))
                    .all(conn)
                    .await?;

                for tag in tag_models {
                    tags.push(PostTag {
                        id: tag.id,
                        name: tag.name,
                    });
                }
            }

            // Construct the post with relations
            return Ok(Some(PostWithRelations {
                id: post.id,
                title: post.title,
                slug: post.slug,
                content: post.content,
                excerpt: post.excerpt,
                featured_image: post.featured_image,
                status: post.status,
                published_at: post.published_at,
                created_at: post.created_at,
                updated_at: post.updated_at,
                author_id: post.author_id,
                view_count: post.view_count,
                likes_count: post.likes_count,
                tag_ids: post.tag_ids,
                category,
                tags,
                author: PostAuthor {
                    id: user.id,
                    name: user.name,
                    email: user.email,
                    avatar: user.avatar,
                },
            }));
        }

        Ok(None)
    }

    // Search posts with query parameters
    pub async fn search(conn: &DbConn, query: PostQuery) -> DbResult<(Vec<Model>, u64)> {
        let mut post_query = Self::find();

        // Apply filters
        if let Some(title_filter) = &query.title {
            let title_pattern = format!("%{}%", title_filter);
            post_query = post_query.filter(Column::Title.contains(&title_pattern));
        }

        if let Some(status_filter) = query.status {
            post_query = post_query.filter(Column::Status.eq(status_filter));
        }

        if let Some(author_id_filter) = query.author_id {
            post_query = post_query.filter(Column::AuthorId.eq(author_id_filter));
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

        if let Some(category_id_filter) = query.category_id {
            post_query = post_query.filter(Column::CategoryId.eq(category_id_filter));
        }

        if let Some(search_term) = &query.search {
            let search_pattern = format!("%{}%", search_term);
            post_query = post_query.filter(
                Condition::any()
                    .add(Column::Title.contains(&search_pattern))
                    .add(Column::Content.contains(&search_pattern)),
            );
        }

        if let Some(tag_ids_filter) = query.tag_ids {
            if !tag_ids_filter.is_empty() {
                // Note: This is a simplified approach. For a real application,
                // you might need a more complex query depending on how tags are stored
                post_query = post_query.filter(Expr::cust_with_values(
                    "tag_ids @> Array[?]::int[]",
                    tag_ids_filter,
                ));
            }
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
                    "view_count" => post_query.order_by(Column::ViewCount, order),
                    "likes_count" => post_query.order_by(Column::LikesCount, order),
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
                p.status, p.published_at, p.created_at, p.updated_at, p.author_id,
                u.name as user_name, u.avatar as user_avatar
            FROM posts p
            JOIN users u ON p.author_id = u.id
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

        if let Some(author_id) = query.author_id {
            where_clauses.push("p.author_id = ?");
            params.push(author_id.into());
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
                "view_count" => "p.view_count",
                "likes_count" => "p.likes_count",
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
            Ok(rows) => rows
                .into_iter()
                .map(|row| PostWithUser {
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
                    author_id: row.try_get("", "author_id").unwrap_or_default(),
                    user_name: row.try_get("", "user_name").unwrap_or_default(),
                    user_avatar: row.try_get("", "user_avatar").ok(),
                })
                .collect(),
            Err(err) => return Err(err.into()),
        };

        // Count total posts with the same filters but without pagination
        let mut count_sql = format!(
            "SELECT COUNT(*) as total FROM posts p JOIN users u ON p.author_id = u.id WHERE 1=1"
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
    // Increment post view count and record view
    pub async fn increment_view_count(
        conn: &DbConn,
        post_id: i32,
        user_id: Option<i32>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> DbResult<()> {
        // Create a view record
        let now = chrono::Utc::now().naive_utc();
        let view = super::super::post_view::ActiveModel {
            post_id: Set(post_id),
            user_id: Set(user_id),
            ip_address: Set(ip_address),
            user_agent: Set(user_agent),
            created_at: Set(now),
            ..Default::default()
        };

        // Use a transaction manually
        let transaction = conn.begin().await?;

        // Insert the view
        match view.insert(&transaction).await {
            Ok(_) => {}
            Err(err) => {
                transaction.rollback().await?;
                return Err(err.into());
            }
        }

        // Increment view count in the post
        let post = Self::find_by_id(post_id).one(&transaction).await?;
        if let Some(post_model) = post {
            let mut post_active: ActiveModel = post_model.into();
            post_active.view_count = Set(post_active.view_count.unwrap() + 1);
            match post_active.update(&transaction).await {
                Ok(_) => {}
                Err(err) => {
                    transaction.rollback().await?;
                    return Err(err.into());
                }
            }
        }

        // Commit the transaction
        transaction.commit().await?;
        Ok(())
    }

    pub async fn get_posts_with_stats(
        conn: &DbConn,
        query: PostQuery,
    ) -> DbResult<(Vec<PostWithStats>, u64)> {
        // Custom SQL query to join posts with users and count views and comments
        let sql = r#"
            SELECT
                p.id, p.title, p.slug, p.content, p.excerpt, p.featured_image,
                p.status, p.published_at, p.created_at, p.updated_at, p.author_id,
                u.name as user_name, u.avatar as user_avatar,
                COUNT(DISTINCT pv.id) as view_count,
                COUNT(DISTINCT pc.id) as comment_count
            FROM posts p
            JOIN users u ON p.author_id, = u.id
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

        if let Some(author_id) = query.author_id {
            where_clauses.push("p.author_id = ?");
            params.push(author_id.into());
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
                "likes_count" => "p.likes_count",
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
            Ok(rows) => rows
                .into_iter()
                .map(|row| PostWithStats {
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
                    author_id: row.try_get("", "author_id").unwrap_or_default(),
                    user_name: row.try_get("", "user_name").unwrap_or_default(),
                    user_avatar: row.try_get("", "user_avatar").ok(),
                    view_count: row.try_get("", "view_count").unwrap_or_default(),
                    comment_count: row.try_get("", "comment_count").unwrap_or_default(),
                })
                .collect(),
            Err(err) => return Err(err.into()),
        };

        // Count total posts with the same filters but without pagination
        let mut count_sql = format!(
            "SELECT COUNT(DISTINCT p.id) as total FROM posts p JOIN users u ON p.author_id = u.id WHERE 1=1"
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

    // Find posts with full relations (user, category, tags)
    pub async fn find_with_relations(
        conn: &DbConn,
        query: PostQuery,
        user: Option<&super::super::user::Model>,
        is_mod: bool,
    ) -> DbResult<(Vec<PostWithRelations>, u64)> {
        let mut post_query = Self::find();

        // Apply filters similar to search function
        if let Some(title_filter) = &query.title {
            let pattern = format!("%{}%", title_filter);
            post_query = post_query.filter(Column::Title.contains(&pattern));
        }

        if let Some(status_filter) = query.status {
            post_query = post_query.filter(Column::Status.eq(status_filter));
        }

        // Apply user filtering based on permissions
        if let Some(user_obj) = user {
            if !is_mod {
                // Non-mods can only see their own posts
                post_query = post_query.filter(Column::AuthorId.eq(user_obj.id));
            } else if let Some(author_id_filter) = query.author_id {
                // Mods can filter by any user
                post_query = post_query.filter(Column::AuthorId.eq(author_id_filter));
            }
        }

        if let Some(category_id_filter) = query.category_id {
            post_query = post_query.filter(Column::CategoryId.eq(category_id_filter));
        }

        if let Some(search_term) = &query.search {
            let pattern = format!("%{}%", search_term);
            post_query = post_query.filter(
                Condition::any()
                    .add(Column::Title.contains(&pattern))
                    .add(Column::Content.contains(&pattern)),
            );
        }

        if let Some(tag_ids_filter) = query.tag_ids {
            if !tag_ids_filter.is_empty() {
                // Note: This is a simplified approach. For a real application,
                // you might need a more complex query depending on how tags are stored
                post_query = post_query.filter(Expr::cust_with_values(
                    "tag_ids @> Array[?]::int[]",
                    tag_ids_filter,
                ));
            }
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
                    "created_at" => post_query.order_by(Column::CreatedAt, order),
                    "updated_at" => post_query.order_by(Column::UpdatedAt, order),
                    "published_at" => post_query.order_by(Column::PublishedAt, order),
                    "view_count" => post_query.order_by(Column::ViewCount, order),
                    "likes_count" => post_query.order_by(Column::LikesCount, order),
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

        // Count total before pagination
        let total = post_query.clone().count(conn).await?;

        // Handle pagination
        let page = match query.page_no {
            Some(p) if p > 0 => p,
            _ => 1,
        };

        let posts = post_query
            .paginate(conn, Self::PER_PAGE)
            .fetch_page(page - 1)
            .await?;

        // Now load the related data for each post
        let mut posts_with_relations = Vec::new();

        for post in posts {
            // Get user info
            let user = super::super::user::Entity::find_by_id(post.author_id)
                .one(conn)
                .await?
                .ok_or_else(|| {
                    ErrorResponse::new(ErrorCode::RecordNotFound)
                        .with_message(&format!("User with ID {} not found", post.author_id))
                })?;

            // Get category if present
            let category = if let Some(cat_id) = post.category_id {
                match super::super::category::Entity::find_by_id(cat_id)
                    .one(conn)
                    .await?
                {
                    Some(cat) => Some(PostCategory {
                        id: cat.id,
                        name: cat.name.clone(),
                    }),
                    None => None,
                }
            } else {
                None
            };

            // Get tags
            let mut tags = Vec::new();
            if !post.tag_ids.is_empty() {
                let tag_models = super::super::tag::Entity::find()
                    .filter(super::super::tag::Column::Id.is_in(post.tag_ids.clone()))
                    .all(conn)
                    .await?;

                for tag in tag_models {
                    tags.push(PostTag {
                        id: tag.id,
                        name: tag.name,
                    });
                }
            }

            // Construct the post with relations
            posts_with_relations.push(PostWithRelations {
                id: post.id,
                title: post.title,
                slug: post.slug,
                content: post.content,
                excerpt: post.excerpt,
                featured_image: post.featured_image,
                status: post.status,
                published_at: post.published_at,
                created_at: post.created_at,
                updated_at: post.updated_at,
                author_id: post.author_id,
                view_count: post.view_count,
                likes_count: post.likes_count,
                tag_ids: post.tag_ids,
                category,
                tags,
                author: PostAuthor {
                    id: user.id,
                    name: user.name,
                    email: user.email,
                    avatar: user.avatar,
                },
            });
        }

        Ok((posts_with_relations, total))
    }

    // Get posts for sitemap
    // Find published posts with pagination and relations
    pub async fn find_published_paginated(
        conn: &DbConn,
        page: u64,
    ) -> DbResult<(Vec<PostWithRelations>, u64)> {
        let query = PostQuery {
            page_no: Some(page),
            status: Some(PostStatus::Published),
            title: None,
            author_id: None,
            created_at: None,
            updated_at: None,
            published_at: None,
            sort_by: Some(vec!["updated_at".to_string()]),
            sort_order: Some("desc".to_string()),
            category_id: None,
            search: None,
            tag_ids: None,
        };

        // Using the find_with_relations function which already handles pagination
        Self::find_with_relations(conn, query, None, false).await
    }

    // Find all posts
    pub async fn find_all(conn: &DbConn) -> DbResult<Vec<Model>> {
        match Self::find().all(conn).await {
            Ok(posts) => Ok(posts),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn sitemap(conn: &DbConn) -> DbResult<Vec<PostSitemap>> {
        let published_posts = Self::find()
            .filter(Column::Status.eq(PostStatus::Published))
            .all(conn)
            .await?;

        let sitemaps = published_posts
            .into_iter()
            .map(|post| PostSitemap {
                slug: post.slug,
                updated_at: post.updated_at,
                published_at: post.published_at.unwrap_or(post.created_at),
            })
            .collect();

        Ok(sitemaps)
    }
}
