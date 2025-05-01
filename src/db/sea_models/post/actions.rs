use crate::{db::sea_models::tag, error::{DbResult, ErrorCode, ErrorResponse}};
use sea_orm::{
    entity::prelude::*, Condition, JoinType, Order, QueryOrder, Set, TransactionTrait
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
        // Create base query with all necessary joins
        use super::super::user::Column as UserColumn;
        use super::super::category::Column as CategoryColumn;
        use sea_orm::QuerySelect;

        // Start with a basic select
        let mut query = Entity::find()
            // .select_only()
            // .columns([
            //     Column::Id,
            //     Column::Title,
            //     Column::Slug,
            //     Column::Content,
            //     Column::Excerpt,
            //     Column::FeaturedImage,
            //     Column::Status,
            //     Column::PublishedAt,
            //     Column::CreatedAt,
            //     Column::UpdatedAt,
            //     Column::AuthorId,
            //     Column::ViewCount,
            //     Column::LikesCount,
            //     Column::TagIds,
            //     Column::CategoryId,
            // ])
            // Select author fields
            .column_as(UserColumn::Id, "author_id")
            .column_as(UserColumn::Name, "author_name")
            .column_as(UserColumn::Email, "author_email")
            .column_as(UserColumn::Avatar, "author_avatar")
            // Select category fields if present
            .column_as(CategoryColumn::Id, "category_id")
            .column_as(CategoryColumn::Name, "category_name")
            // Add count of comments as a subquery
            // .expr_as(
            //     Expr::cust("COALESCE((SELECT COUNT(*) FROM post_comments WHERE post_comments.post_id = post.id), 0)"),
            //     "comment_count",
            // )
            // Join with author (inner join - must have an author)
            .join(JoinType::InnerJoin, Relation::User.def())
            // Left join with category (might not have a category)
            .join(JoinType::LeftJoin, Relation::Category.def());

        // Apply filter by ID or slug
        query = match (post_id, post_slug.clone()) {
            (Some(id), _) => query.filter(Column::Id.eq(id)),
            (_, Some(slug)) => query.filter(Column::Slug.eq(slug)),
            _ => return Ok(None),
        };

        // Execute the query and get the post
        let post_result = query.into_model::<PostWithJoinedData>().one(conn).await?;
        
        if let Some(post_data) = post_result {
            // Get tags for this post (separate query since we need to filter by tag_ids array)
            let mut tags = Vec::new();
            // if !post_data.tag_ids.is_empty() {
            //     let tag_models = super::super::tag::Entity::find()
            //         .filter(super::super::tag::Column::Id.is_in(post_data.tag_ids.clone()))
            //         .all(conn)
            //         .await?;

            //     for tag in tag_models {
            //         tags.push(PostTag {
            //             id: tag.id,
            //             name: tag.name,
            //         });
            //     }
            // }

            // Construct the final PostWithRelations from the joined data
            return Ok(Some(PostWithRelations {
                id: post_data.id,
                title: post_data.title,
                slug: post_data.slug,
                content: post_data.content,
                excerpt: post_data.excerpt,
                featured_image: post_data.featured_image,
                status: post_data.status,
                published_at: post_data.published_at,
                created_at: post_data.created_at,
                updated_at: post_data.updated_at,
                author_id: post_data.author_id,
                view_count: post_data.view_count,
                likes_count: post_data.likes_count,
                tag_ids: post_data.tag_ids,
                // Build category from joined data
                category: PostCategory {
                    id: post_data.category_id,
                    name: post_data.category_name,
                },
                // Use tags we loaded
                tags,
                // Build author from joined data
                author: PostAuthor {
                    id: post_data.author_id,
                    name: post_data.author_name,
                    email: post_data.author_email,
                    avatar: post_data.author_avatar,
                },
                // Use the comment count from the subquery
                comment_count: Some(post_data.comment_count),
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

        // Using the find_with_relations function which is reusable for this purpose
        Self::find_with_relations(conn, query).await
    }

    // Helper method for find_published_paginated
    async fn find_with_relations(
        conn: &DbConn,
        query: PostQuery,
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

        if let Some(author_id_filter) = query.author_id {
            post_query = post_query.filter(Column::AuthorId.eq(author_id_filter));
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
            let category = PostCategory {
                        id: post.category_id,
                        name: "jjaa".to_owned(),
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
            
            // Get comment count using ORM
            let comment_count = super::super::post_comment::Entity::find()
                .filter(super::super::post_comment::Column::PostId.eq(post.id))
                .count(conn)
                .await?;

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
                comment_count: Some(comment_count as i64),
            });
        }

        Ok((posts_with_relations, total))
    }

    // Sitemap data for published posts
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
