use std::collections::HashSet;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_macros::debug_handler;
use fake::faker::internet::en::*;
use fake::faker::lorem::en::*;
use fake::faker::lorem::raw as l;
use fake::faker::name::en::*;
use fake::locales::EN;
use rand::seq::IndexedRandom;
use serde_json::json;

#[derive(Debug, Dummy)]
struct FakeWord(#[dummy(faker = "Word()")] String);

use crate::db::models::user::AdminUserQuery;
use crate::{
    db::models::{
        category::{Category, NewCategory},
        post::{NewPost, Post},
        post_comment::{NewPostComment, PostComment},
        tag::{NewTag, Tag},
        user::{AdminCreateUser, User, UserRole},
    },
    services::auth::AuthSession,
    AppState,
};

use fake::{Dummy, Fake, Faker};
use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};

#[derive(Debug, Dummy)]
pub struct FakeUser {
    #[dummy(faker = "Name()")]
    name: String,
    #[dummy(faker = "FreeEmail()")]
    email: String,
}

// #[derive(Debug, Dummy)]
// pub struct FakePost {
//     #[dummy(faker = "Words(1..5)")]
//     title: String,
//     // #[dummy(faker = "Sentences(1..3)")]
//     // title: String,
//     // #[dummy(faker = "Words(1..5)")]
//     // excerpt: String,

//     // #[dummy(faker = "Paragraphs(1..5)")]
//     // content: String,
// }
//
#[debug_handler]
pub async fn seed_tags(State(state): State<AppState>, _auth: AuthSession) -> impl IntoResponse {
    let mut tags: Vec<Tag> = vec![];
    let mut fake_tags_set: HashSet<String> = HashSet::new();

    // Create 50 fake tags
    for _ in 0..50 {
        let fake_tag = l::Word(EN).fake::<String>();
        fake_tags_set.insert(fake_tag);
    }

    for tag in fake_tags_set {
        let new_tag = NewTag {
            name: tag.clone(),
            slug: tag.to_lowercase(),
            description: None,
        };

        match Tag::create(&state.db_pool, new_tag).await {
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
    let mut fakes: Vec<Category> = vec![];
    let mut fake_set: HashSet<String> = HashSet::new();

    for _ in 0..10 {
        let fake = l::Word(EN).fake::<String>();
        fake_set.insert(fake);
    }

    for cat in fake_set {
        let new_cat = NewCategory {
            name: cat.clone(),
            slug: cat.to_lowercase(),
            parent_id: None,
            description: None,
            logo_image: None,
            cover_image: None,
        };

        match Category::create(&state.db_pool, new_cat).await {
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
    let tags = match Tag::find_all(&state.db_pool).await {
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

    let categories = match Category::find_all(&state.db_pool).await {
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

    let mut authors: Vec<User> = vec![];
    let mut author_page: i64 = 1;
    let mut fetch_authors = true;

    loop {
        if fetch_authors {
            let author_query = AdminUserQuery {
                page_no: Some(author_page),
                email: None,
                name: None,
                role: Some(UserRole::Author),
                status: None,
                created_at: None,
                updated_at: None,
                sort_by: None,
                sort_order: None,
            };
            match User::admin_list(&state.db_pool, author_query).await {
                Ok(res) => {
                    let len = res.len() as i64;
                    if len == User::ADMIN_PER_PAGE {
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
            let category_id = categories.choose(&mut rng).map(|c| c.id);
            let tags_amount = rng.random_range(1..10);
            let tag_ids: Vec<i32> = tags
                .choose_multiple(&mut rng, tags_amount)
                .cloned()
                .map(|t| t.id)
                .collect();
            let post_excerpt = l::Words(EN, 1..8).fake::<Vec<String>>().join(" ");
            let post_content: String = l::Paragraphs(EN, 1..8).fake::<Vec<String>>().join(" ");
            let is_published = rng.random_bool(0.8);
            let new_post = NewPost {
                title: post_title.clone(),
                excerpt: Some(post_excerpt),
                content: post_content,
                author_id: user.id,
                published_at: if is_published {
                    Some(chrono::Utc::now().naive_utc())
                } else {
                    None
                },
                is_published,
                slug: post_slug,
                featured_image_url: None,
                category_id,
                view_count: 0,
                likes_count: 0,
                tag_ids,
            };

            // println!("{:?}", new_post);

            if let Err(err) = Post::create(&state.db_pool, new_post).await {
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
    let mut users: Vec<User> = vec![];
    let mut user_page: i64 = 1;
    let mut fetch_users = true;

    loop {
        if fetch_users {
            let user_query = AdminUserQuery {
                page_no: Some(user_page),
                email: None,
                name: None,
                role: Some(UserRole::User),
                status: None,
                created_at: None,
                updated_at: None,
                sort_by: None,
                sort_order: None,
            };
            match User::admin_list(&state.db_pool, user_query).await {
                Ok(res) => {
                    let len = res.len() as i64;
                    if len == User::ADMIN_PER_PAGE {
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
                            "message": "Failed to fetch authors"
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

    let posts = match Post::find_all(&state.db_pool).await {
        Ok(p) => p,
        Err(_) => {
            return (
                StatusCode::OK,
                Json(json!({
                    "message": "Posts seeded successfully",
                })),
            )
                .into_response();
        }
    };
    let mut rng = StdRng::seed_from_u64(100);
    for user in users {
        let posts_ratio = posts.len() / 10;
        let posts_amount = rng.random_range(posts_ratio..posts.len());
        let post_ids: Vec<i32> = posts
            .choose_multiple(&mut rng, posts_amount)
            .cloned()
            .map(|t| t.id)
            .collect();

        for post_id in post_ids {
            let content: String = l::Sentences(EN, 1..5).fake::<Vec<String>>().join(" ");
            let new_comment = NewPostComment {
                post_id,
                user_id: user.id,
                content,
            };
            if let Err(err) = PostComment::create(&state.db_pool, new_comment).await {
                println!("Error creating comment: {:?}", err);
            }
        }
    }

    return (
        StatusCode::OK,
        Json(json!({
            "message": "Posts seeded successfully",
        })),
    )
        .into_response();
}

#[debug_handler]
pub async fn seed(State(state): State<AppState>, _auth: AuthSession) -> impl IntoResponse {
    let mut rng = StdRng::seed_from_u64(42);
    let mut fake_users: Vec<User> = vec![];
    let mut fake_posts: Vec<Post> = vec![];

    for _ in 0..50 {
        let user: FakeUser = Faker.fake_with_rng(&mut rng);
        let new_user = AdminCreateUser {
            name: user.name,
            email: user.email.clone(),
            password: user.email,
            role: if rng.random_bool(0.5) {
                UserRole::Author
            } else {
                UserRole::User
            },
            avatar: None,
            is_verified: Some(true),
        };

        match User::admin_create(&state.db_pool, new_user).await {
            Ok(user) => fake_users.push(user),
            Err(err) => {
                println!("Error creating user: {:?}", err);
            }
        }
    }

    // Create 10 categories
    let mut categories: Vec<Category> = vec![];
    for _ in 0..10 {
        let fake_name: FakeWord = Faker.fake();
        let name = fake_name.0;
        let slug = name.to_lowercase().replace(' ', "-");
        let new_category = NewCategory {
            name,
            slug,
            description: None,
            parent_id: None,
            cover_image: None,
            logo_image: None,
        };

        match Category::create(&state.db_pool, new_category).await {
            Ok(category) => categories.push(category),
            Err(err) => {
                println!("Error creating category: {:?}", err);
            }
        }
    }

    // Create 50 tags
    let mut tags: Vec<Tag> = vec![];
    for _ in 0..50 {
        let fake_name: FakeWord = Faker.fake();
        let name = fake_name.0;
        let slug = name.to_lowercase().replace(' ', "-");
        let new_tag = NewTag {
            name,
            slug,
            description: None,
        };

        match Tag::create(&state.db_pool, new_tag).await {
            Ok(tag) => tags.push(tag),
            Err(err) => {
                println!("Error creating tag: {:?}", err);
            }
        }
    }

    // Create posts for authors
    for user in fake_users.iter() {
        if user.role == UserRole::Author {
            let num_posts = rng.random_range(2..16);
            for _ in 0..num_posts {
                let category_id = categories.choose(&mut rng).map(|c| c.id);
                let tags_amount = rng.random_range(1..4);
                let tag_ids: Vec<i32> = tags
                    .choose_multiple(&mut rng, tags_amount)
                    .cloned()
                    .map(|t| t.id)
                    .collect();
                let post_title: String = l::Sentence(EN, 1..2).fake();
                let post_excerpt = l::Words(EN, 1..8).fake::<Vec<String>>().join(" ");
                let post_content: String = l::Paragraph(EN, 1..8).fake::<String>();
                let new_post = NewPost {
                    title: post_title.clone(),
                    excerpt: Some(post_excerpt),
                    content: post_content,
                    author_id: user.id,
                    published_at: if rng.random_bool(0.5) {
                        Some(chrono::Utc::now().naive_utc())
                    } else {
                        None
                    },
                    is_published: rng.random_bool(0.5),
                    slug: post_title.to_lowercase().replace(' ', "-"),
                    featured_image_url: None,
                    category_id,
                    view_count: 0,
                    likes_count: 0,
                    tag_ids,
                };

                match Post::create(&state.db_pool, new_post).await {
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
        if user.role == UserRole::User {
            let num_comments = rng.random_range(1..4);
            for _ in 0..num_comments {
                let post = fake_posts.choose(&mut rng).unwrap();
                let content: String = l::Sentence(EN, 1..2).fake::<String>();
                let new_comment = NewPostComment {
                    post_id: post.id,
                    user_id: user.id,
                    content,
                };

                match PostComment::create(&state.db_pool, new_comment).await {
                    Ok(_) => {}
                    Err(err) => {
                        println!("Error creating comment: {:?}", err);
                    }
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
