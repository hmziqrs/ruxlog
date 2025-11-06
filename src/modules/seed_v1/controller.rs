use std::collections::HashSet;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_macros::debug_handler;
use fake::faker::internet::en::*;
use fake::faker::lorem::en::*;
use fake::faker::lorem::raw as l;
use fake::faker::name::en::*;
use fake::locales::EN;
use rand::seq::IndexedRandom;
use sea_orm::EntityTrait;
use serde_json::json;

#[derive(Debug, Dummy)]
struct FakeWord(#[dummy(faker = "Word()")] String);

use crate::db::sea_models::user::{self, AdminUserQuery};
use crate::{
    db::sea_models::{category, post, post_comment, tag, user::UserRole},
    services::auth::AuthSession,
    AppState,
};

use fake::{Dummy, Fake, Faker};
use rand::{rngs::StdRng, Rng, SeedableRng};

#[derive(Debug, Dummy)]
pub struct FakeUser {
    #[dummy(faker = "Name()")]
    name: String,
    #[dummy(faker = "FreeEmail()")]
    email: String,
}

#[debug_handler]
pub async fn seed_tags(State(state): State<AppState>, _auth: AuthSession) -> impl IntoResponse {
    let mut tags: Vec<tag::Model> = vec![];
    let mut fake_tags_set: HashSet<String> = HashSet::new();

    for _ in 0..50 {
        let fake_tag = l::Word(EN).fake::<String>();
        fake_tags_set.insert(fake_tag);
    }

    for tag in fake_tags_set {
        let new_tag = tag::NewTag {
            name: tag.clone(),
            slug: tag.to_lowercase(),
            description: None,
            color: Some("#3b82f6".to_string()),
            text_color: None,
            is_active: Some(true),
        };

        match tag::Entity::create(&state.sea_db, new_tag).await {
            Ok(tag) => tags.push(tag),
            Err(err) => {
                println!("Error creating tag: {:?}", err);
            }
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "message": "Tags seeded successfully",
            "data": tags,
        })),
    )
        .into_response()
}

#[debug_handler]
pub async fn seed_categories(
    State(state): State<AppState>,
    _auth: AuthSession,
) -> impl IntoResponse {
    let mut fakes: Vec<category::CategoryWithRelations> = vec![];
    let mut fake_set: HashSet<String> = HashSet::new();

    for _ in 0..10 {
        let fake = l::Word(EN).fake::<String>();
        fake_set.insert(fake);
    }

    for cat in fake_set {
        let new_cat = category::NewCategory {
            name: cat.clone(),
            slug: cat.to_lowercase(),
            parent_id: None,
            description: None,
            logo_id: None,
            cover_id: None,
            color: Some("#3b82f6".to_string()),
            text_color: None,
            is_active: Some(true),
        };

        match category::Entity::create(&state.sea_db, new_cat).await {
            Ok(tag) => fakes.push(tag),
            Err(err) => {
                println!("Error creating tag: {:?}", err);
            }
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "message": "Categories seeded successfully",
            "data": fakes,
        })),
    )
        .into_response()
}

#[debug_handler]
pub async fn seed_posts(State(state): State<AppState>, _auth: AuthSession) -> impl IntoResponse {
    let tags = match tag::Entity::find_all(&state.sea_db).await {
        Ok(t) => t,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Failed to fetch tags"
                })),
            )
                .into_response();
        }
    };

    let categories = match category::Entity::find_all(&state.sea_db).await {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Failed to fetch categories"
                })),
            )
                .into_response();
        }
    };

    let mut authors: Vec<user::UserWithRelations> = vec![];
    let mut author_page: u64 = 1;
    let mut fetch_authors = true;

    loop {
        if fetch_authors {
            let author_query = AdminUserQuery {
                page: Some(author_page),
                email: None,
                name: None,
                role: Some(UserRole::Author),
                status: None,
                sorts: None,
                created_at_gt: None,
                created_at_lt: None,
                updated_at_gt: None,
                updated_at_lt: None,
            };
            match user::Entity::admin_list(&state.sea_db, author_query).await {
                Ok((res, _)) => {
                    let len = res.len() as u64;
                    if len == user::Entity::PER_PAGE {
                        author_page += 1;
                    } else {
                        fetch_authors = false;
                    }

                    authors.extend(res);
                }
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "message": "Failed to fetch authors"
                        })),
                    )
                        .into_response();
                }
            };
        }
        if !fetch_authors {
            break;
        }
    }

    let mut slugs_set: HashSet<String> = HashSet::new();
    let mut rng = StdRng::seed_from_u64(4999);

    for user in authors.iter() {
        let num_posts = rng.random_range(1..16);
        for _ in 0..num_posts {
            let post_title: String = l::Sentences(EN, 2..5).fake::<Vec<String>>().join(" ");
            let post_slug = post_title.to_lowercase().replace(' ', "-");
            if slugs_set.contains(&post_slug) {
                continue;
            } else {
                slugs_set.insert(post_slug.clone());
            }
            let category_id = categories.choose(&mut rng).map(|c| c.id).unwrap();

            let tags_amount = rng.random_range(1..10);
            let tag_ids: Vec<i32> = tags
                .choose_multiple(&mut rng, tags_amount)
                .cloned()
                .map(|t| t.id)
                .collect();
            let post_excerpt = l::Words(EN, 1..8).fake::<Vec<String>>().join(" ");
            let post_content_text: String = l::Paragraphs(EN, 1..8).fake::<Vec<String>>().join(" ");
            let post_content = serde_json::json!({
                "time": chrono::Utc::now().timestamp_millis(),
                "blocks": [
                    {"type": "paragraph", "data": {"text": post_content_text}}
                ],
                "version": "2.30.7"
            });
            let is_published = rng.random_bool(0.8);

            let new_post = post::NewPost {
                title: post_title.clone(),
                slug: post_slug,
                content: post_content,
                excerpt: Some(post_excerpt),
                featured_image: None,
                status: if is_published {
                    post::PostStatus::Published
                } else {
                    post::PostStatus::Draft
                },
                author_id: user.id,
                published_at: if is_published {
                    Some(chrono::Utc::now().fixed_offset())
                } else {
                    None
                },
                category_id,
                view_count: 0,
                likes_count: 0,
                tag_ids,
            };

            if let Err(err) = post::Entity::create(&state.sea_db, new_post).await {
                println!("Error creating post: {:?}", err);
            }
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "message": "Posts seeded successfully",
            "tags": tags,
            "categories": categories,
            "authors": authors,
        })),
    )
        .into_response()
}

#[debug_handler]
pub async fn seed_post_comments(
    State(state): State<AppState>,
    _auth: AuthSession,
) -> impl IntoResponse {
    let mut users: Vec<user::UserWithRelations> = vec![];
    let mut user_page: u64 = 1;
    let mut fetch_users = true;

    loop {
        if fetch_users {
            let user_query = AdminUserQuery {
                page: Some(user_page),
                email: None,
                name: None,
                role: Some(UserRole::User),
                status: None,
                sorts: None,
                created_at_gt: None,
                created_at_lt: None,
                updated_at_gt: None,
                updated_at_lt: None,
            };
            match user::Entity::admin_list(&state.sea_db, user_query).await {
                Ok((res, _)) => {
                    let len = res.len() as u64;
                    if len == user::Entity::PER_PAGE {
                        user_page += 1;
                    } else {
                        fetch_users = false;
                    }

                    users.extend(res);
                }
                Err(_) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "message": "Failed to fetch users"
                        })),
                    )
                        .into_response();
                }
            };
        }
        if !fetch_users {
            break;
        }
    }

    let posts = match post::Entity::find().all(&state.sea_db).await {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Failed to fetch posts"
                })),
            )
                .into_response();
        }
    };
    let mut rng = StdRng::seed_from_u64(100);
    for user in users {
        // Ensure we don't try to select more posts than available
        let posts_amount = if posts.len() <= 10 {
            1
        } else {
            rng.random_range(1..posts.len().min(10))
        };

        let post_ids: Vec<i32> = posts
            .choose_multiple(&mut rng, posts_amount)
            .cloned()
            .map(|t| t.id)
            .collect();

        for post_id in post_ids {
            let content: String = l::Sentences(EN, 1..5).fake::<Vec<String>>().join(" ");
            let new_comment = post_comment::NewComment {
                post_id,
                user_id: user.id,
                // parent_id: None,
                content,
                likes_count: Some(0),
            };
            if let Err(err) = post_comment::Entity::create(&state.sea_db, new_comment).await {
                println!("Error creating comment: {:?}", err);
            }
        }
    }

    return (
        StatusCode::OK,
        Json(json!({
            "message": "Post comments seeded successfully",
        })),
    )
        .into_response();
}

#[debug_handler]
pub async fn seed(State(state): State<AppState>, _auth: AuthSession) -> impl IntoResponse {
    let mut rng = StdRng::seed_from_u64(42);
    let mut fake_users: Vec<user::UserWithRelations> = vec![];
    let mut fake_posts: Vec<post::PostWithRelations> = vec![];

    for _ in 0..50 {
        let user: FakeUser = Faker.fake_with_rng(&mut rng);
        let new_user = user::AdminCreateUser {
            name: user.name,
            email: user.email.clone(),
            password: user.email,
            role: if rng.random_bool(0.1) {
                UserRole::Admin
            } else if rng.random_bool(0.5) {
                UserRole::Author
            } else {
                UserRole::User
            },
            avatar_id: None,
            is_verified: Some(true),
        };

        match user::Entity::admin_create(&state.sea_db, new_user).await {
            Ok(user) => fake_users.push(user),
            Err(err) => {
                println!("Error creating user: {:?}", err);
            }
        }
    }

    let mut categories: Vec<category::CategoryWithRelations> = vec![];
    for _ in 0..10 {
        let fake_name: FakeWord = Faker.fake();
        let name = fake_name.0;
        let slug = name.to_lowercase().replace(' ', "-");
        let new_category = category::NewCategory {
            name,
            slug,
            description: None,
            parent_id: None,
            cover_id: None,
            logo_id: None,
            color: Some("#3b82f6".to_string()),
            text_color: None,
            is_active: Some(true),
        };

        match category::Entity::create(&state.sea_db, new_category).await {
            Ok(category) => categories.push(category),
            Err(err) => {
                println!("Error creating category: {:?}", err);
            }
        }
    }

    let mut tags: Vec<tag::Model> = vec![];
    for _ in 0..50 {
        let fake_name: FakeWord = Faker.fake();
        let name = fake_name.0;
        let slug = name.to_lowercase().replace(' ', "-");
        let new_tag = tag::NewTag {
            name,
            slug,
            description: None,
            color: Some("#3b82f6".to_string()),
            text_color: None,
            is_active: Some(true),
        };

        match tag::Entity::create(&state.sea_db, new_tag).await {
            Ok(tag) => tags.push(tag),
            Err(err) => {
                println!("Error creating tag: {:?}", err);
            }
        }
    }

    for user in fake_users.iter() {
        if user.role == UserRole::Author {
            let num_posts = rng.random_range(2..16);
            for _ in 0..num_posts {
                let category_id = categories.choose(&mut rng).map(|c| c.id).unwrap();
                let tags_amount = rng.random_range(1..4);
                let tag_ids: Vec<i32> = tags
                    .choose_multiple(&mut rng, tags_amount)
                    .cloned()
                    .map(|t| t.id)
                    .collect();
                let post_title: String = l::Sentence(EN, 1..2).fake();
                let post_excerpt = l::Words(EN, 1..8).fake::<Vec<String>>().join(" ");
                let post_content_text: String = l::Paragraph(EN, 1..8).fake();
                let post_content = serde_json::json!({
                    "time": chrono::Utc::now().timestamp_millis(),
                    "blocks": [
                        {"type": "paragraph", "data": {"text": post_content_text}}
                    ],
                    "version": "2.30.7"
                });
                let is_published = rng.random_bool(0.5);

                let new_post = post::NewPost {
                    title: post_title.clone(),
                    slug: post_title.to_lowercase().replace(' ', "-"),
                    content: post_content,
                    excerpt: Some(post_excerpt),
                    featured_image: None,
                    status: if is_published {
                        post::PostStatus::Published
                    } else {
                        post::PostStatus::Draft
                    },
                    author_id: user.id,
                    published_at: if is_published {
                        Some(chrono::Utc::now().fixed_offset())
                    } else {
                        None
                    },
                    category_id,
                    view_count: 0,
                    likes_count: 0,
                    tag_ids,
                };

                match post::Entity::create(&state.sea_db, new_post).await {
                    Ok(post) => {
                        fake_posts.push(post);
                    }
                    Err(err) => {
                        println!("Error creating post: {:?}", err);
                    }
                }
            }
        }
    }

    for user in fake_users.iter() {
        if user.role == UserRole::User && !fake_posts.is_empty() {
            let num_comments = rng.random_range(1..4);
            for _ in 0..num_comments {
                let post = fake_posts.choose(&mut rng).unwrap();
                let content: String = l::Sentence(EN, 1..2).fake();
                let new_comment = post_comment::NewComment {
                    post_id: post.id,
                    user_id: user.id,
                    // parent_id: None,
                    content,
                    likes_count: Some(0),
                };

                if let Err(err) = post_comment::Entity::create(&state.sea_db, new_comment).await {
                    println!("Error creating comment: {:?}", err);
                }
            }
        }
    }

    (
        StatusCode::OK,
        Json(json!({"message": "Data seeded successfully!"})),
    )
        .into_response()
}
