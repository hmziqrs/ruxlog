#![allow(unused)]
#![allow(clippy::all)]

use std::collections::HashMap;

use super::user::User;
use axum::{http::StatusCode, Json};
use chrono::{Duration, NaiveDateTime, Utc};
use deadpool_diesel::postgres::Pool;
use diesel::query_dsl::methods::FindDsl;
use diesel::QueryDsl;
use diesel::{debug_query, prelude::*};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::task;

use crate::db::models::post_view::PostView;
use crate::db::{
    errors::DBError,
    schema,
    utils::{combine_errors, execute_db_operation},
};

#[derive(Queryable, Clone, Debug, Selectable, Identifiable, Serialize, PartialEq)]
#[diesel(table_name = schema::posts)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub author_id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub published_at: Option<NaiveDateTime>,
    pub is_published: bool,
    pub slug: String,
    pub excerpt: Option<String>,
    pub featured_image_url: Option<String>,
    pub category_id: Option<i32>,
    pub view_count: i32,
    pub likes_count: i32,
    pub tag_ids: Vec<i32>,
}

#[derive(Debug, Serialize)]
pub struct PostSitemap {
    pub slug: String,
    pub updated_at: NaiveDateTime,
    pub published_at: NaiveDateTime,
}

#[derive(Debug, Serialize)]
pub struct PostCategory {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct PostTag {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct PostAuthor {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub avatar: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PostWithRelations {
    #[serde(flatten)]
    pub post: Post,
    pub category: Option<PostCategory>,
    pub tags: Vec<PostTag>,
    pub author: PostAuthor,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = schema::posts)]
pub struct NewPost {
    pub title: String,
    pub content: String,
    pub author_id: i32,
    pub published_at: Option<NaiveDateTime>,
    pub is_published: bool,
    pub slug: String,
    pub excerpt: Option<String>,
    pub featured_image_url: Option<String>,
    pub category_id: Option<i32>,
    pub view_count: i32,
    pub likes_count: i32,
    pub tag_ids: Vec<i32>,
}

#[derive(AsChangeset, Deserialize, Debug)]
#[diesel(table_name = schema::posts)]
pub struct UpdatePost {
    pub title: Option<String>,
    pub content: Option<String>,
    pub author_id: Option<i32>,
    pub published_at: Option<Option<NaiveDateTime>>,
    pub updated_at: NaiveDateTime,
    pub is_published: Option<bool>,
    pub slug: Option<String>,
    pub excerpt: Option<Option<String>>,
    pub featured_image_url: Option<Option<String>>,
    pub category_id: Option<Option<i32>>,
    pub view_count: Option<i32>,
    pub likes_count: Option<i32>,
    pub tag_ids: Option<Vec<i32>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PostSortBy {
    Title,
    UpdatedAt,
    PublishedAt,
    ViewCount,
    LikesCount,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct PostQuery {
    pub page_no: Option<i64>,
    pub author_id: Option<i32>,
    pub category_id: Option<i32>,
    pub is_published: Option<bool>,
    pub search: Option<String>,
    pub sort_by: Option<PostSortBy>,
    pub sort_order: Option<String>,
    pub tag_ids: Option<Vec<i32>>,
}

impl Post {
    pub const PER_PAGE: i64 = 16;
    pub async fn find_by_id_or_slug(
        pool: &Pool,
        post_id: Option<i32>,
        post_slug: Option<String>,
    ) -> Result<Option<PostWithRelations>, DBError> {
        use crate::db::schema::posts::dsl::*;
        use crate::db::schema::{categories, posts, tags, users};

        execute_db_operation(pool, move |conn| {
            let mut query_builder = posts::table
                .left_join(categories::table)
                .inner_join(users::table)
                .into_boxed();

            query_builder = match (post_id, post_slug) {
                (Some(post_id), _) => query_builder.filter(posts::dsl::id.eq(post_id)),
                (_, Some(post_slug)) => query_builder.filter(posts::dsl::slug.eq(post_slug)),
                _ => return Ok(None),
            };

            let result: Option<(Post, Option<Category>, User)> = query_builder
                .select((
                    Post::as_select(),
                    Option::<Category>::as_select(),
                    User::as_select(),
                ))
                .first(conn)
                .optional()?;

            if let Some((post, category, author)) = result {
                // let tags_map = HashMap<i32, Tag>::new()
                // Get all relevant tags
                //
                // let tags_map: HashMap<i32, Tag> = if !post.tag_ids.is_empty() {
                //     tags::table
                //         .filter(tags::dsl::id.eq_any(&post.tag_ids))
                //         .load::<Tag>(conn)?
                //         .into_iter()
                //         .map(|tag| (tag.id, tag))
                //         .collect()
                // } else {
                //     HashMap::new()
                // };

                // Transform the result into PostWithRelations
                let post_with_relations = PostWithRelations {
                    post: post.clone(),
                    category: category.map(|c| PostCategory {
                        id: c.id,
                        name: c.name,
                    }),
                    tags: Vec::new(),
                    // tags: post
                    //     .tag_ids
                    //     .iter()
                    //     .filter_map(|&tag_id| {
                    //         tags_map.get(&tag_id).map(|tag| PostTag {
                    //             id: tag.id,
                    //             name: tag.name.clone(),
                    //         })
                    //     })
                    //     .collect(),
                    author: PostAuthor {
                        id: author.id,
                        name: author.name,
                        email: author.email,
                        avatar: author.avatar,
                    },
                };

                Ok(Some(post_with_relations))
            } else {
                Ok(None)
            }
        })
        .await
    }

    pub async fn find_posts_with_query(
        pool: &Pool,
        query: PostQuery,
        user: User,
    ) -> Result<Vec<PostWithRelations>, DBError> {
        use crate::db::schema::{categories, posts, tags, users};

        execute_db_operation(pool, move |conn| {
            let mut query_builder = posts::table
                .left_join(categories::table)
                .inner_join(users::table)
                .into_boxed();

            // Apply existing filters
            // Mod and above can see all posts by all authors
            if user.is_mod() {
                if let Some(author_id_filter) = query.author_id {
                    query_builder = query_builder.filter(posts::author_id.eq(author_id_filter));
                }
            } else {
                query_builder = query_builder.filter(posts::author_id.eq(user.id));
            }

            if let Some(category_id_filter) = query.category_id {
                query_builder = query_builder.filter(posts::category_id.eq(category_id_filter));
            }
            if let Some(is_published_filter) = query.is_published {
                query_builder = query_builder.filter(posts::is_published.eq(is_published_filter));
            }
            if let Some(search_term) = query.search {
                let search_pattern = format!("%{}%", search_term.to_lowercase());
                query_builder = query_builder.filter(
                    posts::title
                        .ilike(search_pattern.clone())
                        .or(posts::content.ilike(search_pattern)),
                );
            }

            if let Some(tag_ids_filter) = query.tag_ids {
                if !tag_ids_filter.is_empty() {
                    query_builder =
                        query_builder.filter(posts::tag_ids.overlaps_with(tag_ids_filter));
                }
            }

            // Apply sorting
            query_builder = match query.sort_by {
                Some(PostSortBy::Title) => query_builder.order(posts::title.asc()),
                Some(PostSortBy::UpdatedAt) => query_builder.order(posts::updated_at.desc()),
                Some(PostSortBy::PublishedAt) => {
                    query_builder.order(posts::published_at.desc().nulls_last())
                }
                Some(PostSortBy::ViewCount) => query_builder.order(posts::view_count.desc()),
                Some(PostSortBy::LikesCount) => query_builder.order(posts::likes_count.desc()),
                None => query_builder.order(posts::created_at.desc()),
            };

            query_builder = match query.sort_order.as_deref() {
                Some("asc") => query_builder.then_order_by(posts::id.asc()),
                _ => query_builder.then_order_by(posts::id.desc()),
            };

            // Apply pagination
            query_builder = query_builder
                .limit(Self::PER_PAGE)
                .offset((query.page_no.unwrap_or(1) - 1) * Self::PER_PAGE);

            // Execute the query and get posts with joined data
            let results: Vec<(Post, Option<Category>, User)> = query_builder
                .select((
                    Post::as_select(),
                    Option::<Category>::as_select(),
                    User::as_select(),
                ))
                .load(conn)?;

            // Get all relevant tags
            let all_tag_ids: Vec<i32> = results
                .iter()
                .flat_map(|(post, _, _)| post.tag_ids.clone())
                .collect();

            // let tags_map: HashMap<i32, Tag> = if !all_tag_ids.is_empty() {
            //     tags::table
            //         .filter(tags::dsl::id.eq_any(all_tag_ids))
            //         .load::<Tag>(conn)?
            //         .into_iter()
            //         .map(|tag| (tag.id, tag))
            //         .collect()
            // } else {
            //     HashMap::new()
            // };

            // Transform the results into PostWithRelations
            let posts_with_relations = results
                .into_iter()
                .map(|(post, category, author)| PostWithRelations {
                    post: post.clone(),
                    category: category.map(|c| PostCategory {
                        id: c.id,
                        name: c.name,
                    }),
                    tags: Vec::new(),
                    // tags: post
                    //     .tag_ids
                    //     .iter()
                    //     .filter_map(|&tag_id| {
                    //         tags_map.get(&tag_id).map(|tag| PostTag {
                    //             id: tag.id,
                    //             name: tag.name.clone(),
                    //         })
                    //     })
                    //     .collect(),
                    author: PostAuthor {
                        id: author.id,
                        name: author.name,
                        email: author.email,
                        avatar: author.avatar,
                    },
                })
                .collect();

            Ok(posts_with_relations)
        })
        .await
    }

    pub async fn find_published_paginated(
        pool: &Pool,
        page: i64,
    ) -> Result<(Vec<PostWithRelations>, i64), DBError> {
        use crate::db::schema::posts::dsl::*;
        use crate::db::schema::{categories, posts, tags, users};

        execute_db_operation(pool, move |conn| {
            let total = posts
                .filter(is_published.eq(true))
                .count()
                .get_result(conn)?;

            let results: Vec<(Post, Option<Category>, User)> = posts::table
                .left_join(categories::table)
                .inner_join(users::table)
                .filter(is_published.eq(true))
                .order(updated_at.desc())
                .limit(Self::PER_PAGE)
                .offset((page - 1) * Self::PER_PAGE)
                .select((
                    Post::as_select(),
                    Option::<Category>::as_select(),
                    User::as_select(),
                ))
                .load(conn)?;

            // Get all relevant tags
            let all_tag_ids: Vec<i32> = results
                .iter()
                .flat_map(|(post, _, _)| post.tag_ids.clone())
                .collect();

            // let tags_map: HashMap<i32, Tag> = if !all_tag_ids.is_empty() {
            //     tags::table
            //         .filter(tags::dsl::id.eq_any(all_tag_ids))
            //         .load::<Tag>(conn)?
            //         .into_iter()
            //         .map(|tag| (tag.id, tag))
            //         .collect()
            // } else {
            //     HashMap::new()
            // };

            // Transform the results into PostWithRelations
            let posts_with_relations = results
                .into_iter()
                .map(|(post, category, author)| PostWithRelations {
                    post: post.clone(),
                    category: category.map(|c| PostCategory {
                        id: c.id,
                        name: c.name,
                    }),
                    tags: Vec::new(),
                    // tags: post
                    //     .tag_ids
                    //     .iter()
                    //     .filter_map(|&tag_id| {
                    //         tags_map.get(&tag_id).map(|tag| PostTag {
                    //             id: tag.id,
                    //             name: tag.name.clone(),
                    //         })
                    //     })
                    //     .collect(),
                    author: PostAuthor {
                        id: author.id,
                        name: author.name,
                        email: author.email,
                        avatar: author.avatar,
                    },
                })
                .collect();

            Ok((posts_with_relations, total))
        })
        .await
    }

    pub async fn create(pool: &Pool, new_post: NewPost) -> Result<Self, DBError> {
        use crate::db::schema::posts::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::insert_into(posts)
                .values(&new_post)
                .get_result(conn)
        })
        .await
    }

    pub async fn update(
        pool: &Pool,
        post_id: i32,
        user: User,
        update_post: UpdatePost,
    ) -> Result<Option<Self>, DBError> {
        use crate::db::schema::posts::dsl::*;

        execute_db_operation(pool, move |conn| {
            if !user.is_mod() {
                return diesel::update(posts.filter(id.eq(post_id).and(author_id.eq(user.id))))
                    .set(&update_post)
                    .returning(Self::as_returning())
                    .get_result(conn)
                    .optional();
            } else {
                return diesel::update(posts.filter(id.eq(post_id)))
                    .set(&update_post)
                    .returning(Self::as_returning())
                    .get_result(conn)
                    .optional();
            }
        })
        .await
    }

    pub async fn delete(pool: &Pool, user: User, post_id: i32) -> Result<usize, DBError> {
        use crate::db::schema::posts::dsl::*;

        execute_db_operation(pool, move |conn| {
            if !user.is_mod() {
                diesel::delete(posts.filter(id.eq(post_id).and(author_id.eq(user.id))))
                    .execute(conn)
            } else {
                diesel::delete(posts.filter(id.eq(post_id))).execute(conn)
            }
        })
        .await
    }

    pub async fn increment_view_count(
        pool: &Pool,
        q_post_id: i32,
        q_user_id: Option<i32>,
    ) -> Result<(), DBError> {
        use crate::db::schema::posts::dsl::*;

        execute_db_operation(pool, move |conn| {
            conn.transaction(|conn| {
                PostView::create_query(conn, q_post_id, q_user_id)?;
                diesel::update(posts.filter(id.eq_all(q_post_id)))
                    .set(view_count.eq(view_count + 1))
                    .execute(conn)
            })
        })
        .await
        .map(|_| ())
    }

    pub async fn sitemap(pool: &Pool) -> Result<Vec<PostSitemap>, DBError> {
        use crate::db::schema::posts::dsl::*;

        let res = execute_db_operation(pool, move |conn| {
            // only select slug, updated_at, and published_at
            crate::db::schema::posts::table
                .filter(is_published.eq(true))
                .select((slug, updated_at, published_at))
                .load::<(String, NaiveDateTime, Option<NaiveDateTime>)>(conn)
                .map(|res| {
                    res.into_iter()
                        .map(|obj| PostSitemap {
                            slug: obj.0,
                            updated_at: obj.1,
                            published_at: obj.2.unwrap_or(obj.1),
                        })
                        .collect()
                })
        })
        .await?;

        Ok(res)
    }

    pub async fn find_all(pool: &Pool) -> Result<Vec<Self>, DBError> {
        use crate::db::schema::posts::dsl::*;

        execute_db_operation(pool, move |conn| posts.load::<Self>(conn)).await
    }
}
