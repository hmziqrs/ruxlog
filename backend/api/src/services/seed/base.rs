use std::collections::HashMap;

use fake::{
    faker::internet::en::FreeEmail, faker::lorem::raw as l, faker::name::en::Name, locales::EN,
    Dummy, Fake, Faker,
};
use rand::{seq::IndexedRandom, Rng};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, Set,
};
use sea_orm::sea_query::Expr;

use crate::db::sea_models::{
    category, comment_flag, email_verification, forgot_password, media, media_usage, media_variant,
    newsletter_subscriber, post, post_comment, post_revision, post_series, post_view, route_status,
    scheduled_post, seed_run, tag, user, user_session,
};
use crate::services::seed_config::SeedMode;

use super::types::{
    compute_range, seeded_rng, ProgressCallback, SeedOutcome, SeedResult, TableRange,
};

#[derive(Debug, Dummy)]
struct FakeWord(#[dummy(faker = "l::Word(EN)")] String);

#[derive(Debug, Dummy)]
struct FakeUser {
    #[dummy(faker = "Name()")]
    name: String,
    #[dummy(faker = "FreeEmail()")]
    email: String,
}

/// Seed everything locally (no Supabase) and record ranges into `seed_runs`.
pub async fn seed_all(db: &DatabaseConnection) -> SeedResult<SeedOutcome> {
    seed_all_with_progress(db, None, None).await
}

/// Seed everything with optional progress callback for TUI
pub async fn seed_all_with_progress(
    db: &DatabaseConnection,
    progress: Option<ProgressCallback>,
    seed_mode: Option<SeedMode>,
) -> SeedResult<SeedOutcome> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let log = |msg: String| {
        if let Some(ref callback) = progress {
            callback(msg);
        }
    };
    // Capture ID state before seeding for tracking.
    let before_users = user::Entity::find()
        .order_by_desc(user::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let before_categories = category::Entity::find()
        .order_by_desc(category::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let before_tags = tag::Entity::find()
        .order_by_desc(tag::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let before_posts = post::Entity::find()
        .order_by_desc(post::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let before_post_comments = post_comment::Entity::find()
        .order_by_desc(post_comment::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let before_user_sessions = user_session::Entity::find()
        .order_by_desc(user_session::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let before_email_verifications = email_verification::Entity::find()
        .order_by_desc(email_verification::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let before_forgot_passwords = forgot_password::Entity::find()
        .order_by_desc(forgot_password::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let before_post_revisions = post_revision::Entity::find()
        .order_by_desc(post_revision::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let before_post_series = post_series::Entity::find()
        .order_by_desc(post_series::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let before_post_views = post_view::Entity::find()
        .order_by_desc(post_view::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let before_scheduled_posts = scheduled_post::Entity::find()
        .order_by_desc(scheduled_post::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let before_media = media::Entity::find()
        .order_by_desc(media::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let before_media_variants = media_variant::Entity::find()
        .order_by_desc(media_variant::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let before_media_usage = media_usage::Entity::find()
        .order_by_desc(media_usage::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let before_comment_flags = comment_flag::Entity::find()
        .order_by_desc(comment_flag::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let before_newsletter_subscribers = newsletter_subscriber::Entity::find()
        .order_by_desc(newsletter_subscriber::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let before_route_status = route_status::Entity::find()
        .order_by_desc(route_status::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);

    let seed_mode = seed_mode.unwrap_or_default();
    let seed_value = seed_mode.to_seed();
    log(format!("Using seed: {}", seed_value));
    let mut rng = seeded_rng(Some(seed_mode));
    let mut fake_users: Vec<user::UserWithRelations> = vec![];
    let mut fake_posts: Vec<post::PostWithRelations> = vec![];

    log("Creating users (0/50)...".to_string());
    for i in 0..50 {
        let user: FakeUser = Faker.fake_with_rng(&mut rng);
        let email = user.email.clone();
        let password = user.email.clone();
        let new_user = user::AdminCreateUser {
            name: user.name,
            email: email.clone(),
            password: password.clone(),
            role: if rng.random_bool(0.1) {
                user::UserRole::Admin
            } else if rng.random_bool(0.5) {
                user::UserRole::Author
            } else {
                user::UserRole::User
            },
            avatar_id: None,
            is_verified: Some(true),
        };

        match user::Entity::admin_create(db, new_user).await {
            Ok(user) => {
                fake_users.push(user);
                if (i + 1) % 10 == 0 || i + 1 == 50 {
                    log(format!("Creating users ({}/50)...", i + 1));
                }
            }
            Err(err) => {
                let err_msg = format!("Failed to create user '{}': {}", email, err);
                errors.push(err_msg.clone());
                log(err_msg);
            }
        }
    }

    let mut categories: Vec<category::CategoryWithRelations> = vec![];
    log("Creating categories (0/10)...".to_string());
    for i in 0..10 {
        let fake_name: FakeWord = Faker.fake();
        let name = fake_name.0;
        let slug = name.to_lowercase().replace(' ', "-");
        let new_category = category::NewCategory {
            name: name.clone(),
            slug,
            description: None,
            parent_id: None,
            cover_id: None,
            logo_id: None,
            color: Some("#3b82f6".to_string()),
            text_color: None,
            is_active: Some(true),
        };

        match category::Entity::create(db, new_category).await {
            Ok(category) => {
                categories.push(category);
                if (i + 1) % 5 == 0 || i + 1 == 10 {
                    log(format!("Creating categories ({}/10)...", i + 1));
                }
            }
            Err(err) => {
                let err_msg = format!("Failed to create category '{}': {}", name, err);
                errors.push(err_msg.clone());
                log(err_msg);
            }
        }
    }

    let mut tags: Vec<tag::Model> = vec![];
    log("Creating tags (0/50)...".to_string());
    for i in 0..50 {
        let fake_name: FakeWord = Faker.fake();
        let name = fake_name.0;
        let slug = name.to_lowercase().replace(' ', "-");
        let new_tag = tag::NewTag {
            name: name.clone(),
            slug,
            description: None,
            color: Some("#3b82f6".to_string()),
            text_color: None,
            is_active: Some(true),
        };

        match tag::Entity::create(db, new_tag).await {
            Ok(tag) => {
                tags.push(tag);
                if (i + 1) % 10 == 0 || i + 1 == 50 {
                    log(format!("Creating tags ({}/50)...", i + 1));
                }
            }
            Err(err) => {
                let err_msg = format!("Failed to create tag '{}': {}", name, err);
                errors.push(err_msg.clone());
                log(err_msg);
            }
        }
    }

    log("Creating posts...".to_string());
    let mut post_count = 0;
    for user in fake_users.iter() {
        if user.role == user::UserRole::Author {
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

                match post::Entity::create(db, new_post).await {
                    Ok(post) => {
                        fake_posts.push(post);
                        post_count += 1;
                        if post_count % 10 == 0 {
                            log(format!("Created {} posts...", post_count));
                        }
                    }
                    Err(err) => {
                        let err_msg = format!("Failed to create post '{}': {}", post_title, err);
                        errors.push(err_msg.clone());
                        log(err_msg);
                    }
                }
            }
        }
    }
    log(format!("Created {} posts total", post_count));

    log("Creating comments...".to_string());
    let mut comment_count = 0;
    for user in fake_users.iter() {
        if user.role == user::UserRole::User && !fake_posts.is_empty() {
            let num_comments = rng.random_range(1..4);
            for _ in 0..num_comments {
                let post = fake_posts.choose(&mut rng).unwrap();
                let content: String = l::Sentence(EN, 1..2).fake();
                let new_comment = post_comment::NewComment {
                    post_id: post.id,
                    user_id: user.id,
                    content: content.clone(),
                    likes_count: Some(0),
                };

                match post_comment::Entity::create(db, new_comment).await {
                    Ok(_) => {
                        comment_count += 1;
                        if comment_count % 20 == 0 {
                            log(format!("Created {} comments...", comment_count));
                        }
                    }
                    Err(err) => {
                        let err_msg = format!("Failed to create comment: {}", err);
                        errors.push(err_msg.clone());
                        log(err_msg);
                    }
                }
            }
        }
    }
    log(format!("Created {} comments total", comment_count));

    log("Seeding user sessions...".to_string());
    if let Err(e) = seed_user_sessions(db).await {
        warnings.push(format!("Failed to seed user sessions: {}", e));
    }

    log("Seeding email verifications...".to_string());
    if let Err(e) = seed_email_verifications(db).await {
        warnings.push(format!("Failed to seed email verifications: {}", e));
    }

    log("Seeding forgot passwords...".to_string());
    if let Err(e) = seed_forgot_passwords(db).await {
        warnings.push(format!("Failed to seed forgot passwords: {}", e));
    }

    log("Seeding post revisions...".to_string());
    if let Err(e) = seed_post_revisions(db).await {
        warnings.push(format!("Failed to seed post revisions: {}", e));
    }

    log("Seeding post series...".to_string());
    if let Err(e) = seed_post_series(db).await {
        warnings.push(format!("Failed to seed post series: {}", e));
    }

    log("Seeding post views...".to_string());
    if let Err(e) = seed_post_views(db).await {
        warnings.push(format!("Failed to seed post views: {}", e));
    }

    log("Seeding scheduled posts...".to_string());
    if let Err(e) = seed_scheduled_posts(db).await {
        warnings.push(format!("Failed to seed scheduled posts: {}", e));
    }

    log("Seeding media...".to_string());
    if let Err(e) = seed_media(db).await {
        warnings.push(format!("Failed to seed media: {}", e));
    }

    log("Seeding media variants...".to_string());
    if let Err(e) = seed_media_variants(db).await {
        warnings.push(format!("Failed to seed media variants: {}", e));
    }

    log("Seeding media usage...".to_string());
    if let Err(e) = seed_media_usage(db).await {
        warnings.push(format!("Failed to seed media usage: {}", e));
    }

    log("Seeding comment flags...".to_string());
    if let Err(e) = seed_comment_flags(db).await {
        warnings.push(format!("Failed to seed comment flags: {}", e));
    }

    log("Seeding newsletter subscribers...".to_string());
    if let Err(e) = seed_newsletter_subscribers(db).await {
        warnings.push(format!("Failed to seed newsletter subscribers: {}", e));
    }

    log("Seeding route status...".to_string());
    if let Err(e) = seed_route_status(db).await {
        warnings.push(format!("Failed to seed route status: {}", e));
    }

    log("Additional data seeded".to_string());

    // Capture ID state after seeding and record the ranges.
    let after_users = user::Entity::find()
        .order_by_desc(user::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let after_categories = category::Entity::find()
        .order_by_desc(category::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let after_tags = tag::Entity::find()
        .order_by_desc(tag::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let after_posts = post::Entity::find()
        .order_by_desc(post::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let after_post_comments = post_comment::Entity::find()
        .order_by_desc(post_comment::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let after_user_sessions = user_session::Entity::find()
        .order_by_desc(user_session::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let after_email_verifications = email_verification::Entity::find()
        .order_by_desc(email_verification::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let after_forgot_passwords = forgot_password::Entity::find()
        .order_by_desc(forgot_password::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let after_post_revisions = post_revision::Entity::find()
        .order_by_desc(post_revision::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let after_post_series = post_series::Entity::find()
        .order_by_desc(post_series::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let after_post_views = post_view::Entity::find()
        .order_by_desc(post_view::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let after_scheduled_posts = scheduled_post::Entity::find()
        .order_by_desc(scheduled_post::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let after_media = media::Entity::find()
        .order_by_desc(media::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let after_media_variants = media_variant::Entity::find()
        .order_by_desc(media_variant::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let after_media_usage = media_usage::Entity::find()
        .order_by_desc(media_usage::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let after_comment_flags = comment_flag::Entity::find()
        .order_by_desc(comment_flag::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let after_newsletter_subscribers = newsletter_subscriber::Entity::find()
        .order_by_desc(newsletter_subscriber::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);
    let after_route_status = route_status::Entity::find()
        .order_by_desc(route_status::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);

    let mut ranges: HashMap<String, TableRange> = HashMap::new();
    ranges.insert(
        "users".to_string(),
        compute_range(before_users, after_users),
    );
    ranges.insert(
        "categories".to_string(),
        compute_range(before_categories, after_categories),
    );
    ranges.insert("tags".to_string(), compute_range(before_tags, after_tags));
    ranges.insert(
        "posts".to_string(),
        compute_range(before_posts, after_posts),
    );
    ranges.insert(
        "post_comments".to_string(),
        compute_range(before_post_comments, after_post_comments),
    );
    ranges.insert(
        "user_sessions".to_string(),
        compute_range(before_user_sessions, after_user_sessions),
    );
    ranges.insert(
        "email_verifications".to_string(),
        compute_range(before_email_verifications, after_email_verifications),
    );
    ranges.insert(
        "forgot_passwords".to_string(),
        compute_range(before_forgot_passwords, after_forgot_passwords),
    );
    ranges.insert(
        "post_revisions".to_string(),
        compute_range(before_post_revisions, after_post_revisions),
    );
    ranges.insert(
        "post_series".to_string(),
        compute_range(before_post_series, after_post_series),
    );
    ranges.insert(
        "post_views".to_string(),
        compute_range(before_post_views, after_post_views),
    );
    ranges.insert(
        "scheduled_posts".to_string(),
        compute_range(before_scheduled_posts, after_scheduled_posts),
    );
    ranges.insert(
        "media".to_string(),
        compute_range(before_media, after_media),
    );
    ranges.insert(
        "media_variants".to_string(),
        compute_range(before_media_variants, after_media_variants),
    );
    ranges.insert(
        "media_usage".to_string(),
        compute_range(before_media_usage, after_media_usage),
    );
    ranges.insert(
        "comment_flags".to_string(),
        compute_range(before_comment_flags, after_comment_flags),
    );
    ranges.insert(
        "newsletter_subscribers".to_string(),
        compute_range(before_newsletter_subscribers, after_newsletter_subscribers),
    );
    ranges.insert(
        "route_status".to_string(),
        compute_range(before_route_status, after_route_status),
    );

    log("Finalizing seed run...".to_string());
    let seed_run_record = seed_run::ActiveModel {
        key: Set("seed".to_string()),
        ranges: Set(
            SeedOutcome {
                ranges: ranges.clone(),
                seed_run_id: None,
                errors: vec![],
                warnings: vec![],
            }
            .ranges_json(),
        ),
        ..Default::default()
    };
    let inserted = seed_run_record.insert(db).await.ok();
    let seed_run_id = inserted.map(|m| m.id);

    if !errors.is_empty() {
        log(format!("Completed with {} errors", errors.len()));
    }
    if !warnings.is_empty() {
        log(format!("Completed with {} warnings", warnings.len()));
    }
    if errors.is_empty() && warnings.is_empty() {
        log("Seed completed successfully!".to_string());
    }

    Ok(SeedOutcome {
        ranges,
        seed_run_id,
        errors,
        warnings,
    })
}

// --- Internal helpers copied from the original seed controller for reuse ---

async fn seed_user_sessions(db: &DatabaseConnection) -> SeedResult<()> {
    let users = user::Entity::find().all(db).await?;
    let devices = vec![
        "MacOS · Chrome 126",
        "Windows · Edge 125",
        "iPhone · Safari 17",
        "Android · Chrome 125",
    ];
    let ip_addresses = vec!["192.168.1.100", "10.0.0.50", "172.16.0.25", "203.0.113.1"];
    let mut rng = seeded_rng(None);

    for user in users {
        let session_count = rng.random_range(1..4);
        for _ in 0..session_count {
            let last_seen = chrono::Utc::now().fixed_offset()
                - chrono::Duration::hours(rng.random_range(1..720));
            let new_session = user_session::Model {
                id: 0,
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

            let active_model = user_session::ActiveModel {
                id: ActiveValue::NotSet,
                user_id: Set(new_session.user_id),
                device: Set(new_session.device),
                ip_address: Set(new_session.ip_address),
                last_seen: Set(new_session.last_seen),
                revoked_at: Set(new_session.revoked_at),
            };

            let _ = active_model.insert(db).await;
        }
    }

    Ok(())
}

async fn seed_email_verifications(db: &DatabaseConnection) -> SeedResult<()> {
    let users = user::Entity::find().all(db).await?;
    let mut rng = seeded_rng(None);
    for user in users {
        let code = email_verification::Entity::generate_code();
        // Spread creation times a bit for realism
        let created_at = chrono::Utc::now().fixed_offset()
            - chrono::Duration::minutes(rng.random_range(0..90));

        let verification = email_verification::Model {
            id: 0,
            user_id: user.id,
            code,
            created_at,
            updated_at: created_at,
        };

        let active_model = email_verification::ActiveModel {
            id: ActiveValue::NotSet,
            user_id: Set(verification.user_id),
            created_at: Set(verification.created_at),
            code: Set(verification.code),
            updated_at: Set(verification.updated_at),
        };

        let _ = active_model.insert(db).await;
    }
    Ok(())
}

async fn seed_forgot_passwords(db: &DatabaseConnection) -> SeedResult<()> {
    let users = user::Entity::find().all(db).await?;
    let mut rng = seeded_rng(None);

    for user in users {
        if rng.random_bool(0.3) {
            let code = forgot_password::Entity::generate_code();
            let created_at = chrono::Utc::now().fixed_offset()
                - chrono::Duration::minutes(rng.random_range(0..60));

            let forgot = forgot_password::Model {
                id: 0,
                user_id: user.id,
                code,
                created_at,
                updated_at: created_at,
            };

            let active_model = forgot_password::ActiveModel {
                id: ActiveValue::NotSet,
                user_id: Set(forgot.user_id),
                created_at: Set(forgot.created_at),
                updated_at: Set(forgot.updated_at),
                code: Set(forgot.code),
            };

            let _ = active_model.insert(db).await;
        }
    }
    Ok(())
}

async fn seed_post_revisions(db: &DatabaseConnection) -> SeedResult<()> {
    let posts = post::Entity::find().all(db).await?;
    let mut rng = seeded_rng(None);

    for post in posts {
        let revision_count = rng.random_range(1..4);
        for _ in 0..revision_count {
            let revision_content: String = l::Paragraph(EN, 1..6).fake_with_rng(&mut rng);
            let revision = post_revision::Model {
                id: 0,
                post_id: post.id,
                content: revision_content.clone(),
                metadata: None,
                created_at: chrono::Utc::now().fixed_offset(),
            };

            let active_model = post_revision::ActiveModel {
                id: ActiveValue::NotSet,
                post_id: Set(revision.post_id),
                content: Set(revision.content),
                metadata: Set(revision.metadata),
                created_at: Set(revision.created_at),
            };

            let _ = active_model.insert(db).await;
        }
    }
    Ok(())
}

async fn seed_post_series(db: &DatabaseConnection) -> SeedResult<()> {
    let mut rng = seeded_rng(None);
    for _ in 0..5 {
        let words: Vec<String> = l::Words(EN, 1..4).fake_with_rng(&mut rng);
        let title = words.join(" ");
        let description: String = l::Sentence(EN, 4..8).fake_with_rng(&mut rng);

        let series = post_series::Model {
            id: 0,
            name: title.clone(),
            slug: title.to_lowercase().replace(' ', "-"),
            description: Some(description),
            created_at: chrono::Utc::now().fixed_offset(),
            updated_at: chrono::Utc::now().fixed_offset(),
        };

        let active_model = post_series::ActiveModel {
            id: ActiveValue::NotSet,
            name: Set(series.name.clone()),
            slug: Set(series.slug.clone()),
            description: Set(series.description.clone()),
            created_at: Set(series.created_at),
            updated_at: Set(series.updated_at),
        };

        let _ = active_model.insert(db).await;
    }

    Ok(())
}

async fn seed_post_views(db: &DatabaseConnection) -> SeedResult<()> {
    let posts = post::Entity::find().all(db).await?;
    let users = user::Entity::find().all(db).await?;
    let mut rng = seeded_rng(None);
    let user_agents = vec![
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)",
        "Mozilla/5.0 (Linux; Android 14)",
    ];
    let ips = vec!["203.0.113.10", "198.51.100.42", "10.0.0.24", "172.16.1.15"];

    for post in posts.into_iter().take(100) {
        let view_count = rng.random_range(1..20);
        for _ in 0..view_count {
            let viewer = users.choose(&mut rng).map(|u| u.id);
            let view = post_view::ActiveModel {
                id: ActiveValue::NotSet,
                post_id: Set(post.id),
                ip_address: Set(Some(ips.choose(&mut rng).unwrap().to_string())),
                user_agent: Set(Some(user_agents.choose(&mut rng).unwrap().to_string())),
                user_id: Set(viewer),
                created_at: Set(chrono::Utc::now().fixed_offset()),
            };
            let _ = view.insert(db).await;

            let _ = post::Entity::update_many()
                .col_expr(
                    post::Column::ViewCount,
                    Expr::col(post::Column::ViewCount).add(1),
                )
                .filter(post::Column::Id.eq(post.id))
                .exec(db)
                .await;
        }
    }

    Ok(())
}

async fn seed_scheduled_posts(db: &DatabaseConnection) -> SeedResult<()> {
    let posts = post::Entity::find()
        .filter(post::Column::Status.eq(post::PostStatus::Draft))
        .all(db)
        .await?;
    let mut rng = seeded_rng(None);

    for post in posts.into_iter().take(20) {
        let scheduled_at = chrono::Utc::now().fixed_offset()
            + chrono::Duration::hours(rng.random_range(24..240));
        let scheduled = scheduled_post::Model {
            id: 0,
            post_id: post.id,
            publish_at: scheduled_at,
            status: scheduled_post::ScheduledPostStatus::Pending,
            created_at: chrono::Utc::now().fixed_offset(),
            updated_at: chrono::Utc::now().fixed_offset(),
        };

        let active_model = scheduled_post::ActiveModel {
            id: ActiveValue::NotSet,
            post_id: Set(scheduled.post_id),
            publish_at: Set(scheduled.publish_at),
            status: Set(scheduled.status),
            created_at: Set(scheduled.created_at),
            updated_at: Set(scheduled.updated_at),
        };

        let _ = active_model.insert(db).await;
    }
    Ok(())
}

async fn seed_media(db: &DatabaseConnection) -> SeedResult<()> {
    let mut rng = seeded_rng(None);
    let media_types = vec!["image/png", "image/jpeg", "image/webp"];
    let sizes = vec![1024, 2048, 4096, 8192];

    for i in 0..50 {
        let mime = media_types.choose(&mut rng).unwrap().to_string();
        let width = Some(rng.random_range(300..2000));
        let height = Some(rng.random_range(300..2000));
        let size = *sizes.choose(&mut rng).unwrap_or(&2048);
        let now = chrono::Utc::now().fixed_offset();

        let media_record = media::Model {
            id: 0,
            object_key: format!("uploads/fake_image_{}.png", i),
            file_url: format!("https://example.com/uploads/fake_image_{}.png", i),
            mime_type: mime,
            width,
            height,
            size,
            extension: Some("png".to_string()),
            uploader_id: None,
            reference_type: None,
            content_hash: None,
            is_optimized: false,
            optimized_at: None,
            created_at: now,
            updated_at: now,
        };

        let active_model = media::ActiveModel {
            id: ActiveValue::NotSet,
            object_key: Set(media_record.object_key),
            file_url: Set(media_record.file_url),
            mime_type: Set(media_record.mime_type),
            width: Set(media_record.width),
            height: Set(media_record.height),
            size: Set(media_record.size),
            extension: Set(media_record.extension),
            uploader_id: Set(media_record.uploader_id),
            reference_type: Set(media_record.reference_type),
            content_hash: Set(media_record.content_hash),
            is_optimized: Set(media_record.is_optimized),
            optimized_at: Set(media_record.optimized_at),
            created_at: Set(media_record.created_at),
            updated_at: Set(media_record.updated_at),
        };

        let _ = active_model.insert(db).await;
    }

    Ok(())
}

async fn seed_media_variants(db: &DatabaseConnection) -> SeedResult<()> {
    let media_items = media::Entity::find().all(db).await?;

    for media_item in media_items.into_iter().take(30) {
        let variants = vec![
            (format!("{}-thumb", media_item.object_key), 200, 200, 32, "thumbnail"),
            (format!("{}-small", media_item.object_key), 400, 400, 64, "small"),
            (format!("{}-medium", media_item.object_key), 800, 800, 128, "medium"),
        ];

        for (key, w, h, size, variant_type) in variants {
            let now = chrono::Utc::now().fixed_offset();
            let variant = media_variant::Model {
                id: 0,
                media_id: media_item.id,
                object_key: key.clone(),
                mime_type: media_item.mime_type.clone(),
                width: Some(w),
                height: Some(h),
                size,
                extension: Some("png".to_string()),
                quality: Some(80),
                variant_type: variant_type.to_string(),
                created_at: now,
                updated_at: now,
            };

            let active_model = media_variant::ActiveModel {
                id: ActiveValue::NotSet,
                media_id: Set(variant.media_id),
                object_key: Set(variant.object_key),
                mime_type: Set(variant.mime_type),
                width: Set(variant.width),
                height: Set(variant.height),
                size: Set(variant.size),
                extension: Set(variant.extension),
                quality: Set(variant.quality),
                variant_type: Set(variant.variant_type),
                created_at: Set(variant.created_at),
                updated_at: Set(variant.updated_at),
            };

            let _ = active_model.insert(db).await;
        }
    }

    Ok(())
}

async fn seed_media_usage(db: &DatabaseConnection) -> SeedResult<()> {
    let media_items = media::Entity::find().all(db).await?;
    let posts = post::Entity::find().all(db).await?;
    let mut rng = seeded_rng(None);

    for media_item in media_items.into_iter().take(10) {
        if let Some(post) = posts.choose(&mut rng) {
            let usage = media_usage::Model {
                id: 0,
                media_id: media_item.id,
                entity_type: media_usage::EntityType::Post,
                entity_id: post.id,
                field_name: "featured_image".to_string(),
                created_at: chrono::Utc::now().fixed_offset(),
            };

            let active_model = media_usage::ActiveModel {
                id: ActiveValue::NotSet,
                media_id: Set(usage.media_id),
                entity_type: Set(usage.entity_type),
                entity_id: Set(usage.entity_id),
                field_name: Set(usage.field_name),
                created_at: Set(usage.created_at),
            };

            let _ = active_model.insert(db).await;
        }
    }

    Ok(())
}

async fn seed_comment_flags(db: &DatabaseConnection) -> SeedResult<()> {
    let comments = post_comment::Entity::find().all(db).await?;
    let users = user::Entity::find().all(db).await?;

    let mut rng = seeded_rng(None);
    let flag_reasons = vec!["spam", "inappropriate", "off-topic", "harassment"];

    for comment in comments.into_iter().take(10) {
        if rng.random_bool(0.3) {
            let flag_user = users.choose(&mut rng).unwrap();
            let reason = flag_reasons.choose(&mut rng).unwrap();

            let flag = comment_flag::Model {
                id: 0,
                comment_id: comment.id,
                user_id: flag_user.id,
                reason: Some(reason.to_string()),
                created_at: chrono::Utc::now().fixed_offset(),
            };

            let active_model = comment_flag::ActiveModel {
                id: ActiveValue::NotSet,
                comment_id: Set(flag.comment_id),
                user_id: Set(flag.user_id),
                reason: Set(flag.reason),
                created_at: Set(flag.created_at),
            };

            let _ = active_model.insert(db).await;
        }
    }

    Ok(())
}

async fn seed_newsletter_subscribers(db: &DatabaseConnection) -> SeedResult<()> {
    let mut subscribers: Vec<newsletter_subscriber::Model> = vec![];
    let mut emails_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut rng = seeded_rng(None);

    for _ in 0..100 {
        let email = FreeEmail().fake::<String>();
        if emails_set.insert(email.clone()) {
            let status = if rng.random_bool(0.85) {
                newsletter_subscriber::SubscriberStatus::Confirmed
            } else if rng.random_bool(0.1) {
                newsletter_subscriber::SubscriberStatus::Unsubscribed
            } else {
                newsletter_subscriber::SubscriberStatus::Pending
            };
            let token: String = (0..12)
                .map(|_| ((rng.random::<u8>() % 26) + b'a') as char)
                .collect();
            let now = chrono::Utc::now().fixed_offset();

            let subscriber = newsletter_subscriber::Model {
                id: 0,
                email,
                status,
                token,
                created_at: now,
                updated_at: now,
            };

            let active_model = newsletter_subscriber::ActiveModel {
                id: ActiveValue::NotSet,
                email: Set(subscriber.email),
                status: Set(subscriber.status),
                token: Set(subscriber.token),
                created_at: Set(subscriber.created_at),
                updated_at: Set(subscriber.updated_at),
            };

            if let Ok(sub) = active_model.insert(db).await {
                subscribers.push(sub);
            }
        }
    }
    Ok(())
}

async fn seed_route_status(db: &DatabaseConnection) -> SeedResult<()> {
    let protected_routes = vec![
        "/admin",
        "/admin/users",
        "/admin/settings",
        "/api/internal",
        "/debug",
    ];

    for route in protected_routes {
        let route_status_entry = route_status::Model {
            id: 0,
            route_pattern: route.to_string(),
            is_blocked: false,
            reason: None,
            created_at: chrono::Utc::now().fixed_offset(),
            updated_at: chrono::Utc::now().fixed_offset(),
        };

        let active_model = route_status::ActiveModel {
            id: ActiveValue::NotSet,
            route_pattern: Set(route_status_entry.route_pattern),
            is_blocked: Set(route_status_entry.is_blocked),
            reason: Set(route_status_entry.reason),
            created_at: Set(route_status_entry.created_at),
            updated_at: Set(route_status_entry.updated_at),
        };

        let _ = active_model.insert(db).await;
    }
    Ok(())
}
