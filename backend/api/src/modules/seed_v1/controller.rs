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
use crate::db::sea_models::scheduled_post::ScheduledPostStatus;
use crate::{
    db::sea_models::{
        category, comment_flag, email_verification, forgot_password, media, media_usage,
        media_variant, newsletter_subscriber, post, post_comment, post_revision, post_series,
        post_view, route_status, scheduled_post, tag, user_session,
        user::UserRole,
    },
    services::auth::AuthSession,
    AppState,
};

use fake::{Dummy, Fake, Faker};
use rand::{rngs::StdRng, Rng, SeedableRng};
use sea_orm::{ActiveModelTrait, Set};

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
                featured_image_id: None,
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

// Authentication related seeds
#[debug_handler]
pub async fn seed_user_sessions(State(state): State<AppState>, _auth: AuthSession) -> impl IntoResponse {
    let users = match user::Entity::find().all(&state.sea_db).await {
        Ok(u) => u,
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

    let devices = vec!["MacOS · Chrome 126", "Windows · Edge 125", "iPhone · Safari 17", "Android · Chrome 125"];
    let ip_addresses = vec!["192.168.1.100", "10.0.0.50", "172.16.0.25", "203.0.113.1"];
    let mut rng = StdRng::seed_from_u64(999);

    for user in users {
        let session_count = rng.random_range(1..4);
        for _ in 0..session_count {
            let last_seen = chrono::Utc::now().fixed_offset() - chrono::Duration::hours(rng.random_range(1..720));
            let new_session = user_session::Model {
                id: 0, // Auto-increment
                user_id: user.id,
                device: Some(devices.choose(&mut rng).unwrap().to_string()),
                ip_address: Some(ip_addresses.choose(&mut rng).unwrap().to_string()),
                last_seen,
                revoked_at: if rng.random_bool(0.2) {
                    Some(last_seen + chrono::Duration::hours(rng.random_range(1..48)))
                } else {
                    None
                },
            };

            // Convert to ActiveModel and insert
            let active_model = user_session::ActiveModel {
                id: Set(new_session.id),
                user_id: Set(new_session.user_id),
                device: Set(new_session.device),
                ip_address: Set(new_session.ip_address),
                last_seen: Set(new_session.last_seen),
                revoked_at: Set(new_session.revoked_at),
            };

            if let Err(err) = active_model.insert(&state.sea_db).await {
                println!("Error creating user session: {:?}", err);
            }
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "message": "User sessions seeded successfully"
        })),
    )
        .into_response()
}

#[debug_handler]
pub async fn seed_email_verifications(State(state): State<AppState>, _auth: AuthSession) -> impl IntoResponse {
    let users = match user::Entity::find().all(&state.sea_db).await {
        Ok(u) => u,
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

    let mut rng = StdRng::seed_from_u64(1234);

    for user in users.into_iter().take(20) {
        let verification = email_verification::Model {
            id: 0, // Auto-increment
            user_id: user.id,
            code: email_verification::Entity::generate_code(),
            created_at: chrono::Utc::now().fixed_offset(),
            updated_at: chrono::Utc::now().fixed_offset(),
        };

        let active_model = email_verification::ActiveModel {
            id: Set(verification.id),
            user_id: Set(verification.user_id),
            code: Set(verification.code),
            created_at: Set(verification.created_at),
            updated_at: Set(verification.updated_at),
        };

        if let Err(err) = active_model.insert(&state.sea_db).await {
            println!("Error creating email verification: {:?}", err);
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "message": "Email verifications seeded successfully"
        })),
    )
        .into_response()
}

#[debug_handler]
pub async fn seed_forgot_passwords(State(state): State<AppState>, _auth: AuthSession) -> impl IntoResponse {
    let users = match user::Entity::find().all(&state.sea_db).await {
        Ok(u) => u,
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

    let mut rng = StdRng::seed_from_u64(5678);

    for user in users.into_iter().take(10) {
        let forgot_password = forgot_password::Model {
            id: 0, // Auto-increment
            user_id: user.id,
            code: forgot_password::Entity::generate_code(),
            created_at: chrono::Utc::now().fixed_offset(),
            updated_at: chrono::Utc::now().fixed_offset(),
        };

        let active_model = forgot_password::ActiveModel {
            id: Set(forgot_password.id),
            user_id: Set(forgot_password.user_id),
            code: Set(forgot_password.code),
            created_at: Set(forgot_password.created_at),
            updated_at: Set(forgot_password.updated_at),
        };

        if let Err(err) = active_model.insert(&state.sea_db).await {
            println!("Error creating forgot password: {:?}", err);
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "message": "Forgot passwords seeded successfully"
        })),
    )
        .into_response()
}

// Post-related seeds
#[debug_handler]
pub async fn seed_post_revisions(State(state): State<AppState>, _auth: AuthSession) -> impl IntoResponse {
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

    let mut rng = StdRng::seed_from_u64(3456);

    for post in posts.into_iter().take(30) {
        let revision_count = rng.random_range(1..5);
        for i in 0..revision_count {
            let content_text = l::Paragraphs(EN, 1..3).fake::<Vec<String>>().join(" ");
            let post_content = serde_json::json!({
                "time": chrono::Utc::now().timestamp_millis(),
                "blocks": [
                    {"type": "paragraph", "data": {"text": content_text}}
                ],
                "version": "2.30.7"
            });

            let revision = post_revision::Model {
                id: 0, // Auto-increment
                post_id: post.id,
                content: post_content.to_string(),
                metadata: Some(serde_json::json!({
                    "title": format!("{} (Revision {})", post.title, i + 1)
                })),
                created_at: chrono::Utc::now().fixed_offset() - chrono::Duration::hours(i as i64 * 24),
            };

            let active_model = post_revision::ActiveModel {
                id: Set(revision.id),
                post_id: Set(revision.post_id),
                content: Set(revision.content),
                metadata: Set(revision.metadata),
                created_at: Set(revision.created_at),
            };

            if let Err(err) = active_model.insert(&state.sea_db).await {
                println!("Error creating post revision: {:?}", err);
            }
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "message": "Post revisions seeded successfully"
        })),
    )
        .into_response()
}

#[debug_handler]
pub async fn seed_post_series(State(state): State<AppState>, _auth: AuthSession) -> impl IntoResponse {
    let mut series_list: Vec<post_series::Model> = vec![];
    let series_names = vec![
        "Getting Started with Rust",
        "Web Development Best Practices",
        "Database Design Patterns",
        "API Security Guide",
        "Frontend Frameworks Comparison",
    ];

    for (i, name) in series_names.iter().enumerate() {
        let new_series = post_series::Model {
            id: 0, // Auto-increment
            name: name.to_string(),
            slug: name.to_lowercase().replace(' ', "-"),
            description: Some(format!("A comprehensive series about {}", name)),
            created_at: chrono::Utc::now().fixed_offset(),
            updated_at: chrono::Utc::now().fixed_offset(),
        };

        let active_model = post_series::ActiveModel {
            id: Set(new_series.id),
            name: Set(new_series.name),
            slug: Set(new_series.slug),
            description: Set(new_series.description),
            created_at: Set(new_series.created_at),
            updated_at: Set(new_series.updated_at),
        };

        match active_model.insert(&state.sea_db).await {
            Ok(series) => series_list.push(series),
            Err(err) => println!("Error creating post series: {:?}", err),
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "message": "Post series seeded successfully",
            "data": series_list
        })),
    )
        .into_response()
}

#[debug_handler]
pub async fn seed_post_views(State(state): State<AppState>, _auth: AuthSession) -> impl IntoResponse {
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

    let users = match user::Entity::find().all(&state.sea_db).await {
        Ok(u) => u,
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

    let mut rng = StdRng::seed_from_u64(7890);
    let ip_addresses = vec!["192.168.1.100", "10.0.0.50", "172.16.0.25", "203.0.113.1"];

    for post in posts {
        let view_count = rng.random_range(50..500);
        for _ in 0..view_count {
            let user_id = if rng.random_bool(0.7) {
                Some(users.choose(&mut rng).map(|u| u.id).unwrap())
            } else {
                None
            };

            let view = post_view::Model {
                id: 0, // Auto-increment
                post_id: post.id,
                user_id,
                ip_address: Some(ip_addresses.choose(&mut rng).unwrap().to_string()),
                user_agent: Some("Mozilla/5.0 (compatible; RuxlogBot/1.0)".to_string()),
                created_at: chrono::Utc::now().fixed_offset() - chrono::Duration::minutes(rng.random_range(1..4320)),
            };

            let active_model = post_view::ActiveModel {
                id: Set(view.id),
                post_id: Set(view.post_id),
                user_id: Set(view.user_id),
                ip_address: Set(view.ip_address),
                user_agent: Set(view.user_agent),
                created_at: Set(view.created_at),
            };

            if let Err(err) = active_model.insert(&state.sea_db).await {
                println!("Error creating post view: {:?}", err);
            }
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "message": "Post views seeded successfully"
        })),
    )
        .into_response()
}

#[debug_handler]
pub async fn seed_scheduled_posts(State(state): State<AppState>, _auth: AuthSession) -> impl IntoResponse {
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

    let mut rng = StdRng::seed_from_u64(9999);

    for post in posts.into_iter().take(10) {
        let scheduled_post = scheduled_post::Model {
            id: 0, // Auto-increment
            post_id: post.id,
            publish_at: chrono::Utc::now().fixed_offset() + chrono::Duration::days(rng.random_range(1..30)),
            status: ScheduledPostStatus::Pending,
            created_at: chrono::Utc::now().fixed_offset(),
            updated_at: chrono::Utc::now().fixed_offset(),
        };

        let active_model = scheduled_post::ActiveModel {
            id: Set(scheduled_post.id),
            post_id: Set(scheduled_post.post_id),
            publish_at: Set(scheduled_post.publish_at),
            status: Set(scheduled_post.status),
            created_at: Set(scheduled_post.created_at),
            updated_at: Set(scheduled_post.updated_at),
        };

        if let Err(err) = active_model.insert(&state.sea_db).await {
            println!("Error creating scheduled post: {:?}", err);
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "message": "Scheduled posts seeded successfully"
        })),
    )
        .into_response()
}

// Media related seeds
#[debug_handler]
pub async fn seed_media(State(state): State<AppState>, _auth: AuthSession) -> impl IntoResponse {
    let users = match user::Entity::find().all(&state.sea_db).await {
        Ok(u) => u,
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

    let mut media_list: Vec<media::Model> = vec![];
    let fake_files = vec![
        ("blog-image-1.jpg", "image/jpeg", Some(1920), Some(1080), 245760),
        ("profile-pic-1.png", "image/png", Some(400), Some(400), 65536),
        ("thumbnail-1.webp", "image/webp", Some(300), Some(200), 12288),
        ("document-1.pdf", "application/pdf", None, None, 102400),
        ("video-thumbnail-1.jpg", "image/jpeg", Some(1280), Some(720), 512000),
    ];

    let mut rng = StdRng::seed_from_u64(7777);

    for (i, (filename, mime_type, width, height, size)) in fake_files.iter().enumerate() {
        let new_media = media::Model {
            id: 0, // Auto-increment
            object_key: format!("uploads/{}", filename),
            file_url: format!("https://cdn.example.com/{}", filename),
            mime_type: mime_type.to_string(),
            width: *width,
            height: *height,
            size: *size,
            extension: Some(filename.split('.').last().unwrap().to_string()),
            uploader_id: Some(users.choose(&mut rng).map(|u| u.id).unwrap()),
            reference_type: Some([media::MediaReference::Post, media::MediaReference::User, media::MediaReference::Category]
                .choose(&mut rng).unwrap().clone()),
            content_hash: Some(format!("hash_{}", i)),
            is_optimized: rng.random_bool(0.6),
            optimized_at: if rng.random_bool(0.6) {
                Some(chrono::Utc::now().fixed_offset())
            } else {
                None
            },
            created_at: chrono::Utc::now().fixed_offset(),
            updated_at: chrono::Utc::now().fixed_offset(),
        };

        let active_model = media::ActiveModel {
            id: Set(new_media.id),
            object_key: Set(new_media.object_key),
            file_url: Set(new_media.file_url),
            mime_type: Set(new_media.mime_type),
            width: Set(new_media.width),
            height: Set(new_media.height),
            size: Set(new_media.size),
            extension: Set(new_media.extension),
            uploader_id: Set(new_media.uploader_id),
            reference_type: Set(new_media.reference_type),
            content_hash: Set(new_media.content_hash),
            is_optimized: Set(new_media.is_optimized),
            optimized_at: Set(new_media.optimized_at),
            created_at: Set(new_media.created_at),
            updated_at: Set(new_media.updated_at),
        };

        match active_model.insert(&state.sea_db).await {
            Ok(media) => media_list.push(media),
            Err(err) => println!("Error creating media: {:?}", err),
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "message": "Media seeded successfully",
            "data": media_list
        })),
    )
        .into_response()
}

#[debug_handler]
pub async fn seed_media_variants(State(state): State<AppState>, _auth: AuthSession) -> impl IntoResponse {
    let media_files = match media::Entity::find().all(&state.sea_db).await {
        Ok(m) => m,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Failed to fetch media files"
                })),
            )
                .into_response();
        }
    };

    let mut rng = StdRng::seed_from_u64(8888);
    let variant_types = vec!["thumbnail", "medium", "large", "webp"];

    for media_item in media_files {
        if media_item.mime_type.starts_with("image/") {
            for variant_type in variant_types.iter().take(rng.random_range(1..4)) {
                let (width, height) = match variant_type {
                    &"thumbnail" => (150, 150),
                    &"medium" => (800, 600),
                    &"large" => (1200, 900),
                    &"webp" => (media_item.width.unwrap_or(800), media_item.height.unwrap_or(600)),
                    _ => (400, 300),
                };

                let variant = media_variant::Model {
                    id: 0, // Auto-increment
                    media_id: media_item.id,
                    variant_type: variant_type.to_string(),
                    width: Some(width),
                    height: Some(height),
                    size: media_item.size / 2, // Assume compressed
                    object_key: format!("variants/{}/{}_{}", media_item.object_key, variant_type, media_item.extension.as_ref().unwrap_or(&"jpg".to_string())),
                    mime_type: if *variant_type == "webp" { "image/webp".to_string() } else { media_item.mime_type.clone() },
                    extension: media_item.extension.clone(),
                    quality: Some(80),
                    created_at: chrono::Utc::now().fixed_offset(),
                    updated_at: chrono::Utc::now().fixed_offset(),
                };

                let active_model = media_variant::ActiveModel {
                    id: Set(variant.id),
                    media_id: Set(variant.media_id),
                    variant_type: Set(variant.variant_type),
                    width: Set(variant.width),
                    height: Set(variant.height),
                    size: Set(variant.size),
                    object_key: Set(variant.object_key),
                    mime_type: Set(variant.mime_type),
                    extension: Set(variant.extension),
                    quality: Set(variant.quality),
                    created_at: Set(variant.created_at),
                    updated_at: Set(variant.updated_at),
                };

                if let Err(err) = active_model.insert(&state.sea_db).await {
                    println!("Error creating media variant: {:?}", err);
                }
            }
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "message": "Media variants seeded successfully"
        })),
    )
        .into_response()
}

#[debug_handler]
pub async fn seed_media_usage(State(state): State<AppState>, _auth: AuthSession) -> impl IntoResponse {
    let media_files = match media::Entity::find().all(&state.sea_db).await {
        Ok(m) => m,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Failed to fetch media files"
                })),
            )
                .into_response();
        }
    };

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

    let users = match user::Entity::find().all(&state.sea_db).await {
        Ok(u) => u,
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

    let categories = match category::Entity::find().all(&state.sea_db).await {
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

    let mut rng = StdRng::seed_from_u64(9999);

    for media_item in media_files {
        let usage_count = rng.random_range(1..4);
        for _ in 0..usage_count {
            let (entity_type, entity_id) = match rng.random_range(0..3) {
                0 => (media_usage::EntityType::Post, posts.choose(&mut rng).map(|p| p.id).unwrap()),
                1 => (media_usage::EntityType::User, users.choose(&mut rng).map(|u| u.id).unwrap()),
                _ => (media_usage::EntityType::Category, categories.choose(&mut rng).map(|c| c.id).unwrap()),
            };

            let usage = media_usage::Model {
                id: 0, // Auto-increment
                media_id: media_item.id,
                entity_type,
                entity_id,
                field_name: "featured_image".to_string(),
                created_at: chrono::Utc::now().fixed_offset(),
            };

            let active_model = media_usage::ActiveModel {
                id: Set(usage.id),
                media_id: Set(usage.media_id),
                entity_type: Set(usage.entity_type),
                entity_id: Set(usage.entity_id),
                field_name: Set(usage.field_name),
                created_at: Set(usage.created_at),
            };

            if let Err(err) = active_model.insert(&state.sea_db).await {
                println!("Error creating media usage: {:?}", err);
            }
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "message": "Media usage records seeded successfully"
        })),
    )
        .into_response()
}

// Community and system seeds
#[debug_handler]
pub async fn seed_comment_flags(State(state): State<AppState>, _auth: AuthSession) -> impl IntoResponse {
    let comments = match post_comment::Entity::find().all(&state.sea_db).await {
        Ok(c) => c,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "message": "Failed to fetch comments"
                })),
            )
                .into_response();
        }
    };

    let users = match user::Entity::find().all(&state.sea_db).await {
        Ok(u) => u,
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

    let mut rng = StdRng::seed_from_u64(1111);
    let flag_reasons = vec!["spam", "inappropriate", "off-topic", "harassment"];

    for comment in comments.into_iter().take(10) {
        if rng.random_bool(0.3) {
            let flag_user = users.choose(&mut rng).unwrap();
            let reason = flag_reasons.choose(&mut rng).unwrap();

            let flag = comment_flag::Model {
                id: 0, // Auto-increment
                comment_id: comment.id,
                user_id: flag_user.id,
                reason: Some(reason.to_string()),
                created_at: chrono::Utc::now().fixed_offset(),
            };

            let active_model = comment_flag::ActiveModel {
                id: Set(flag.id),
                comment_id: Set(flag.comment_id),
                user_id: Set(flag.user_id),
                reason: Set(flag.reason),
                created_at: Set(flag.created_at),
            };

            if let Err(err) = active_model.insert(&state.sea_db).await {
                println!("Error creating comment flag: {:?}", err);
            }
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "message": "Comment flags seeded successfully"
        })),
    )
        .into_response()
}

#[debug_handler]
pub async fn seed_newsletter_subscribers(State(state): State<AppState>, _auth: AuthSession) -> impl IntoResponse {
    let mut subscribers: Vec<newsletter_subscriber::Model> = vec![];
    let mut emails_set: HashSet<String> = HashSet::new();
    let mut rng = StdRng::seed_from_u64(2222);

    for _ in 0..100 {
        let email = SafeEmail().fake::<String>();
        if emails_set.insert(email.clone()) {
            let status = if rng.random_bool(0.85) {
                newsletter_subscriber::SubscriberStatus::Confirmed
            } else if rng.random_bool(0.1) {
                newsletter_subscriber::SubscriberStatus::Pending
            } else {
                newsletter_subscriber::SubscriberStatus::Unsubscribed
            };

            let subscriber = newsletter_subscriber::Model {
                id: 0, // Auto-increment
                email,
                status,
                token: format!("token_{}", rng.random_range(1000..9999)),
                created_at: chrono::Utc::now().fixed_offset(),
                updated_at: chrono::Utc::now().fixed_offset(),
            };

            let active_model = newsletter_subscriber::ActiveModel {
                id: Set(subscriber.id),
                email: Set(subscriber.email),
                status: Set(subscriber.status),
                token: Set(subscriber.token),
                created_at: Set(subscriber.created_at),
                updated_at: Set(subscriber.updated_at),
            };

            match active_model.insert(&state.sea_db).await {
                Ok(sub) => subscribers.push(sub),
                Err(err) => println!("Error creating newsletter subscriber: {:?}", err),
            }
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "message": "Newsletter subscribers seeded successfully",
            "data": subscribers
        })),
    )
        .into_response()
}

#[debug_handler]
pub async fn seed_route_status(State(state): State<AppState>, _auth: AuthSession) -> impl IntoResponse {
    let protected_routes = vec![
        "/admin",
        "/admin/users",
        "/admin/settings",
        "/api/internal",
        "/debug",
    ];

    let mut routes: Vec<route_status::Model> = vec![];

    for route in protected_routes {
        let route_status_entry = route_status::Model {
            id: 0, // Auto-increment
            route_pattern: route.to_string(),
            is_blocked: false,
            reason: None,
            created_at: chrono::Utc::now().fixed_offset(),
            updated_at: chrono::Utc::now().fixed_offset(),
        };

        let active_model = route_status::ActiveModel {
            id: Set(route_status_entry.id),
            route_pattern: Set(route_status_entry.route_pattern),
            is_blocked: Set(route_status_entry.is_blocked),
            reason: Set(route_status_entry.reason),
            created_at: Set(route_status_entry.created_at),
            updated_at: Set(route_status_entry.updated_at),
        };

        match active_model.insert(&state.sea_db).await {
            Ok(r) => routes.push(r),
            Err(err) => println!("Error creating route status: {:?}", err),
        }
    }

    (
        StatusCode::OK,
        Json(json!({
            "message": "Route status records seeded successfully",
            "data": routes
        })),
    )
        .into_response()
}

#[debug_handler]
pub async fn seed(State(state): State<AppState>, _auth: AuthSession) -> impl IntoResponse {
    let mut rng = StdRng::seed_from_u64(42);
    let mut fake_users: Vec<user::UserWithRelations> = vec![];
    let mut fake_posts: Vec<post::PostWithRelations> = vec![];

    for _ in 0..50 {
        let user: FakeUser = Faker.fake_with_rng(&mut rng);
        let email = user.email.clone();
        let password = user.email.clone();
        let new_user = user::AdminCreateUser {
            name: user.name,
            email: email.clone(),
            password: password.clone(),
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
            Ok(user) => {
                // Create user in Supabase
                let supabase = state.supabase.clone();
                let email_clone = email.clone();
                let password_clone = password.clone();
                tokio::spawn(async move {
                    match supabase.admin_create_user(&email_clone, &password_clone).await {
                        Ok(_) => println!("Supabase user created for {}", email_clone),
                        Err(e) => println!("Failed to create Supabase user: {}", e),
                    }
                });

                fake_users.push(user);
            }
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
                    featured_image_id: None,
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

    // Seed additional models
    println!("Seeding user sessions...");
    let _ = seed_user_sessions(State(state.clone()), _auth.clone()).await;

    println!("Seeding email verifications...");
    let _ = seed_email_verifications(State(state.clone()), _auth.clone()).await;

    println!("Seeding forgot passwords...");
    let _ = seed_forgot_passwords(State(state.clone()), _auth.clone()).await;

    println!("Seeding post revisions...");
    let _ = seed_post_revisions(State(state.clone()), _auth.clone()).await;

    println!("Seeding post series...");
    let _ = seed_post_series(State(state.clone()), _auth.clone()).await;

    println!("Seeding post views...");
    let _ = seed_post_views(State(state.clone()), _auth.clone()).await;

    println!("Seeding scheduled posts...");
    let _ = seed_scheduled_posts(State(state.clone()), _auth.clone()).await;

    println!("Seeding media...");
    let _ = seed_media(State(state.clone()), _auth.clone()).await;

    println!("Seeding media variants...");
    let _ = seed_media_variants(State(state.clone()), _auth.clone()).await;

    println!("Seeding media usage...");
    let _ = seed_media_usage(State(state.clone()), _auth.clone()).await;

    println!("Seeding comment flags...");
    let _ = seed_comment_flags(State(state.clone()), _auth.clone()).await;

    println!("Seeding newsletter subscribers...");
    let _ = seed_newsletter_subscribers(State(state.clone()), _auth.clone()).await;

    println!("Seeding route status...");
    let _ = seed_route_status(State(state.clone()), _auth.clone()).await;

    (
        StatusCode::OK,
        Json(json!({"message": "All data seeded successfully! Including users, posts, categories, tags, comments, sessions, email verifications, post revisions, post series, post views, scheduled posts, media, media variants, media usage, comment flags, newsletter subscribers, and route status"})),
    )
        .into_response()
}
