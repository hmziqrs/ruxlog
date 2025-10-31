use std::collections::HashSet;

use crate::{db::sea_models::tag, error::DbResult};
use sea_orm::{
    entity::prelude::*, Condition, JoinType, Order, QueryOrder, QuerySelect, Set, TransactionTrait,
};

use super::*;

impl Entity {
    pub const PER_PAGE: u64 = 10;

    async fn sanitized_tag_ids(conn: &DbConn, tag_ids: Vec<i32>) -> DbResult<Vec<i32>> {
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

    pub async fn delete(conn: &DbConn, post_id: i32) -> DbResult<u64> {
        match Self::delete_by_id(post_id).exec(conn).await {
            Ok(result) => Ok(result.rows_affected),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn find_by_id_or_slug(
        conn: &DbConn,
        post_id: Option<i32>,
        post_slug: Option<String>,
    ) -> DbResult<Option<PostWithRelations>> {
        use super::super::category::Column as CategoryColumn;
        use super::super::user::Column as UserColumn;
        use sea_orm::QuerySelect;
        let mut query = Entity::find()
            .column_as(UserColumn::Id, "author_id")
            .column_as(UserColumn::Name, "author_name")
            .column_as(UserColumn::Email, "author_email")
            .column_as(UserColumn::AvatarId, "author_avatar")
            .column_as(CategoryColumn::Id, "category_id")
            .column_as(CategoryColumn::Name, "category_name")
            .expr_as(
                Expr::cust("COALESCE((SELECT COUNT(*) FROM post_comments WHERE post_comments.post_id = posts.id), 0)"),
                "comment_count",
            )
            .join(JoinType::InnerJoin, Relation::User.def())
            .join(JoinType::LeftJoin, Relation::Category.def());

        query = match (post_id, post_slug.clone()) {
            (Some(id), _) => query.filter(Column::Id.eq(id)),
            (_, Some(slug)) => query.filter(Column::Slug.eq(slug)),
            _ => return Ok(None),
        };

        let post_result = query.into_model::<PostWithJoinedData>().one(conn).await?;

        if let Some(post_data) = post_result {
            let mut tags = Vec::new();
            if !post_data.tag_ids.is_empty() {
                let tag_models = super::super::tag::Entity::find()
                    .filter(super::super::tag::Column::Id.is_in(post_data.tag_ids.clone()))
                    .all(conn)
                    .await?;

                for tag in tag_models {
                    tags.push(PostTag {
                        id: tag.id,
                        name: tag.name,
                    });
                }
            }

            return Ok(Some(post_data.into_relation(tags)));
        }

        Ok(None)
    }

    // Search posts with query parameters and optionally load relations
    pub async fn search(
        conn: &DbConn,
        query: PostQuery,
    ) -> DbResult<(Vec<PostWithRelations>, u64)> {
        let mut post_query = Self::find();

        // If relations are needed, join the related tables
        use super::super::category::Column as CategoryColumn;
        use super::super::user::Column as UserColumn;

        post_query = post_query
            .column_as(UserColumn::Id, "author_id")
            .column_as(UserColumn::Name, "author_name")
            .column_as(UserColumn::Email, "author_email")
            .column_as(UserColumn::AvatarId, "author_avatar")
            .column_as(CategoryColumn::Id, "category_id")
            .column_as(CategoryColumn::Name, "category_name")
            .expr_as(
                Expr::cust("COALESCE((SELECT COUNT(*) FROM post_comments WHERE post_comments.post_id = posts.id), 0)"),
                "comment_count",
            )
            .join(JoinType::InnerJoin, Relation::User.def())
            .join(JoinType::LeftJoin, Relation::Category.def());

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

        // Date range filters
        if let Some(ts) = query.created_at_gt {
            post_query = post_query.filter(Column::CreatedAt.gt(ts));
        }
        if let Some(ts) = query.created_at_lt {
            post_query = post_query.filter(Column::CreatedAt.lt(ts));
        }
        if let Some(ts) = query.updated_at_gt {
            post_query = post_query.filter(Column::UpdatedAt.gt(ts));
        }
        if let Some(ts) = query.updated_at_lt {
            post_query = post_query.filter(Column::UpdatedAt.lt(ts));
        }
        if let Some(ts) = query.published_at_gt {
            post_query = post_query.filter(Column::PublishedAt.gt(ts));
        }
        if let Some(ts) = query.published_at_lt {
            post_query = post_query.filter(Column::PublishedAt.lt(ts));
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
                // Convert the Vec<i32> to a formatted string for PostgreSQL array containment
                let tag_ids_str = tag_ids_filter
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<String>>()
                    .join(",");

                post_query = post_query.filter(Expr::cust(format!(
                    "posts.tag_ids && ARRAY[{}]::int[]",
                    tag_ids_str
                )));
            }
        }

        // Multi-field sorting with per-field order
        if let Some(sorts) = query.sorts {
            for sort in sorts {
                let column = match sort.field.as_str() {
                    "title" => Some(Column::Title),
                    "status" => Some(Column::Status),
                    "created_at" => Some(Column::CreatedAt),
                    "updated_at" => Some(Column::UpdatedAt),
                    "published_at" => Some(Column::PublishedAt),
                    "view_count" => Some(Column::ViewCount),
                    "likes_count" => Some(Column::LikesCount),
                    _ => None,
                };
                if let Some(col) = column {
                    post_query = post_query.order_by(col, sort.order);
                }
            }
        } else {
            post_query = post_query.order_by(Column::CreatedAt, Order::Desc);
        }

        let page = match query.page_no {
            Some(p) if p > 0 => p,
            _ => 1,
        };

        let paginated = post_query
            .into_model::<PostWithJoinedData>()
            .paginate(conn, Self::PER_PAGE);

        let total = paginated.num_items().await?;

        let posts_joined = paginated.fetch_page(page - 1).await?;

        // Collect all tag IDs from each post into a set
        let all_tag_ids: HashSet<i32> = posts_joined
            .iter()
            .flat_map(|p| p.tag_ids.clone())
            .collect();

        // Load all tags in a single query
        let tags = if !all_tag_ids.is_empty() {
            super::super::tag::Entity::find()
                .filter(
                    super::super::tag::Column::Id
                        .is_in(all_tag_ids.into_iter().collect::<Vec<i32>>()),
                )
                .all(conn)
                .await?
                .into_iter()
                .map(|t| {
                    (
                        t.id,
                        PostTag {
                            id: t.id,
                            name: t.name,
                        },
                    )
                })
                .collect::<std::collections::HashMap<i32, PostTag>>()
        } else {
            std::collections::HashMap::new()
        };

        // Map joined data to PostWithRelations
        let posts_with_relations: Vec<PostWithRelations> = posts_joined
            .into_iter()
            .map(|joined_data| {
                let post_tags = joined_data
                    .tag_ids
                    .iter()
                    .filter_map(|id| tags.get(id).cloned())
                    .collect::<Vec<PostTag>>();

                // Convert joined data to PostWithRelations
                joined_data.into_relation(post_tags)
            })
            .collect();

        Ok((posts_with_relations, total))
    }

    pub async fn find_published_paginated(
        conn: &DbConn,
        query: PostQuery,
    ) -> DbResult<(Vec<PostWithRelations>, u64)> {
        let query = PostQuery {
            page_no: query.page_no,
            status: Some(PostStatus::Published),
            title: None,
            author_id: query.author_id,
            sorts: Some(vec![crate::utils::SortParam {
                field: "updated_at".to_string(),
                order: sea_orm::Order::Desc,
            }]),
            category_id: query.category_id,
            search: None,
            tag_ids: query.tag_ids,
            created_at_gt: None,
            created_at_lt: None,
            updated_at_gt: None,
            updated_at_lt: None,
            published_at_gt: None,
            published_at_lt: None,
        };

        Self::search(conn, query).await
    }

    // Sitemap data for published posts
    pub async fn sitemap(conn: &DbConn) -> DbResult<Vec<PostSitemap>> {
        let sitemaps = Self::find()
            .select_only()
            .columns(vec![Column::Slug, Column::UpdatedAt, Column::PublishedAt])
            .filter(Column::Status.eq(PostStatus::Published))
            .into_model::<PostSitemap>()
            .all(conn)
            .await?;

        Ok(sitemaps)
    }

    pub async fn increment_view_count(
        conn: &DbConn,
        post_id: i32,
        user_id: Option<i32>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> DbResult<()> {
        let now = chrono::Utc::now().fixed_offset();
        let view = super::super::post_view::ActiveModel {
            post_id: Set(post_id),
            user_id: Set(user_id),
            ip_address: Set(ip_address),
            user_agent: Set(user_agent),
            created_at: Set(now),
            ..Default::default()
        };

        let transaction = conn.begin().await?;

        // Insert the view
        match view.insert(&transaction).await {
            Ok(_) => {}
            Err(err) => {
                transaction.rollback().await?;
                return Err(err.into());
            }
        }

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
}
