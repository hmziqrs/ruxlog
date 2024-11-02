use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use fake::faker::internet::en::*;
use fake::faker::lorem::en::*;
use fake::faker::lorem::raw as l;
use fake::faker::name::en::*;
use serde_json::json;

#[derive(Debug, Dummy)]
struct FakeWord(#[dummy(faker = "Word()")] String);

use crate::{
    db::models::{
        category::{Category, NewCategory},
        post::{NewPost, Post, PostSortBy},
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

#[debug_handler]
pub async fn seed(State(state): State<AppState>, auth: AuthSession) -> impl IntoResponse {
    let mut rng = StdRng::seed_from_u64(42);
    let mut fake_users: Vec<User> = vec![];
    let mut fake_posts: Vec<Post> = vec![];

    // Create 50 fake users (25 users, 25 authors)
    for _ in 0..50 {
        let user: FakeUser = Faker.fake_with_rng(&mut rng);
        let new_user = AdminCreateUser {
            name: user.name,
            email: user.email,
            password: user.email,
            role: if rng.gen_bool(0.5) {
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
            let num_posts = rng.gen_range(2..16);
            for _ in 0..num_posts {
                // let post: FakePost = Faker.fake_with_rng(&mut rng);
                let category_id = categories.choose(&mut rng).map(|c| c.id).or(None);
                let tag_ids: Vec<i32> = tags
                    .choose_multiple(&mut rng, rng.gen_range(1..4))
                    .iter()
                    .map(|t| t.id)
                    .collect();
                let post_title = l::Sentence(fake::locales::EN, 1..2).fake();
                let post_excerpt = l::Words(fake::locales::EN, 1..8).fake();
                let post_content = l::Paragraphs(fake::locales::EN, 1..8).fake();
                let new_post = NewPost {
                    title: post_title,
                    excerpt: Some(post_excerpt),
                    content: post_content,
                    author_id: user.id,
                    published_at: if rng.gen_bool(0.5) {
                        Some(chrono::Utc::now().naive_utc())
                    } else {
                        None
                    },
                    is_published: rng.gen_bool(0.5),
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

    // // Create comments for users
    // let posts = Post::find_posts_with_query(
    //     &state.db_pool,
    //     PostQuery {
    //         page_no: None,
    //         author_id: None,
    //         category_id: None,
    //         is_published: None,
    //         search: None,
    //         sort_by: Some(PostSortBy::PublishedAt),
    //         sort_order: None,
    //         tag_ids: None,
    //     },
    //     auth.user.unwrap(),
    // )
    // .await;

    // let posts = match posts {
    //     Ok(posts) => posts,
    //     Err(err) => {
    //         println!("Error fetching posts: {:?}", err);
    //         return (
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             Json(json!({
    //                 "error": err.to_string(),
    //                 "message": "Failed to seed data",
    //             })),
    //         )
    //             .into_response();
    //     }
    // };

    for user in fake_users.iter() {
        if user.role == UserRole::User {
            let num_comments = rng.gen_range(1..4);
            for _ in 0..num_comments {
                let post = fake_posts.choose(&mut rng).unwrap();
                let content = Faker.fake::<Sentence>();
                let new_comment = NewPostComment {
                    post_id: post.post.id,
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
