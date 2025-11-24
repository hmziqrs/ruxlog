use std::collections::HashMap;

use fake::{
    faker::{internet::en::FreeEmail, lorem::raw as l, name::en::Name},
    locales::EN,
    Fake,
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
    scheduled_post, tag, user, user_session,
};
use crate::services::seed_config::{CustomSeedTarget, SeedMode, SeedSizePreset};

use super::types::{
    compute_range, seeded_rng, size_label, ProgressCallback, SeedError, SeedOutcome, SeedResult,
    TableRange,
};

pub async fn seed_custom(
    db: &DatabaseConnection,
    target: CustomSeedTarget,
    size: SeedSizePreset,
    seed_mode: Option<SeedMode>,
    custom_count: Option<u32>,
    progress: Option<ProgressCallback>,
) -> SeedResult<SeedOutcome> {
    let mut ranges = HashMap::new();
    let errors = Vec::new();
    let warnings = Vec::new();
    let log = |msg: String| {
        if let Some(ref callback) = progress {
            callback(msg);
        }
    };

    let count = custom_count.unwrap_or_else(|| size.count_for_target(target));
    let range = match target {
        CustomSeedTarget::Users => seed_extra_users(db, count, seed_mode, &log).await?,
        CustomSeedTarget::Categories => seed_extra_categories(db, count, seed_mode, &log).await?,
        CustomSeedTarget::Tags => seed_extra_tags(db, count, seed_mode, &log).await?,
        CustomSeedTarget::Posts => seed_extra_posts(db, count, seed_mode, &log).await?,
        CustomSeedTarget::PostComments => seed_extra_comments(db, count, seed_mode, &log).await?,
        CustomSeedTarget::CommentFlags => {
            seed_extra_comment_flags(db, count, seed_mode, &log).await?
        }
        CustomSeedTarget::PostViews => seed_extra_post_views(db, count, seed_mode, &log).await?,
        CustomSeedTarget::UserSessions => {
            seed_extra_user_sessions(db, count, seed_mode, &log).await?
        }
        CustomSeedTarget::EmailVerifications => {
            seed_extra_email_verifications(db, count, seed_mode, &log).await?
        }
        CustomSeedTarget::ForgotPasswords => {
            seed_extra_forgot_passwords(db, count, seed_mode, &log).await?
        }
        CustomSeedTarget::PostRevisions => {
            seed_extra_post_revisions(db, count, seed_mode, &log).await?
        }
        CustomSeedTarget::PostSeries => seed_extra_post_series(db, count, seed_mode, &log).await?,
        CustomSeedTarget::ScheduledPosts => {
            seed_extra_scheduled_posts(db, count, seed_mode, &log).await?
        }
        CustomSeedTarget::Media => seed_extra_media(db, count, seed_mode, &log).await?,
        CustomSeedTarget::MediaVariants => {
            seed_extra_media_variants(db, count, seed_mode, &log).await?
        }
        CustomSeedTarget::MediaUsage => seed_extra_media_usage(db, count, seed_mode, &log).await?,
        CustomSeedTarget::NewsletterSubscribers => {
            seed_extra_newsletter_subscribers(db, count, seed_mode, &log).await?
        }
        CustomSeedTarget::RouteStatus => {
            seed_extra_route_status(db, count, seed_mode, &log).await?
        }
    };

    ranges.insert(
        match target {
            CustomSeedTarget::Users => "users",
            CustomSeedTarget::Categories => "categories",
            CustomSeedTarget::Tags => "tags",
            CustomSeedTarget::Posts => "posts",
            CustomSeedTarget::PostComments => "post_comments",
            CustomSeedTarget::CommentFlags => "comment_flags",
            CustomSeedTarget::PostViews => "post_views",
            CustomSeedTarget::UserSessions => "user_sessions",
            CustomSeedTarget::EmailVerifications => "email_verifications",
            CustomSeedTarget::ForgotPasswords => "forgot_passwords",
            CustomSeedTarget::PostRevisions => "post_revisions",
            CustomSeedTarget::PostSeries => "post_series",
            CustomSeedTarget::ScheduledPosts => "scheduled_posts",
            CustomSeedTarget::Media => "media",
            CustomSeedTarget::MediaVariants => "media_variants",
            CustomSeedTarget::MediaUsage => "media_usage",
            CustomSeedTarget::NewsletterSubscribers => "newsletter_subscribers",
            CustomSeedTarget::RouteStatus => "route_status",
        }
        .to_string(),
        range,
    );

    Ok(SeedOutcome {
        ranges,
        seed_run_id: None,
        errors,
        warnings,
    })
}

async fn seed_extra_users<F>(
    db: &DatabaseConnection,
    count: u32,
    seed_mode: Option<SeedMode>,
    log: &F,
) -> SeedResult<TableRange>
where
    F: Fn(String),
{
    let mut rng = seeded_rng(seed_mode);
    let before_users = user::Entity::find()
        .order_by_desc(user::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);

    let roles = [user::UserRole::Admin, user::UserRole::Author, user::UserRole::User];

    for i in 0..count {
        let name: String = Name().fake_with_rng(&mut rng);
        let email = FreeEmail().fake_with_rng::<String, _>(&mut rng);
        let role = roles.choose(&mut rng).cloned().unwrap_or(user::UserRole::User);
        let new_user = user::AdminCreateUser {
            name: name.clone(),
            email: email.clone(),
            password: email.clone(),
            role,
            avatar_id: None,
            is_verified: Some(true),
        };

        if let Err(err) = user::Entity::admin_create(db, new_user).await {
            log(format!("Failed to create user {}: {}", email, err));
        } else if (i + 1) % 10 == 0 || i + 1 == count {
            log(format!("Created {} / {} users", i + 1, count));
        }
    }

    let after_users = user::Entity::find()
        .order_by_desc(user::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(before_users);

    Ok(compute_range(before_users, after_users))
}

async fn seed_extra_categories<F>(
    db: &DatabaseConnection,
    count: u32,
    seed_mode: Option<SeedMode>,
    log: &F,
) -> SeedResult<TableRange>
where
    F: Fn(String),
{
    let mut rng = seeded_rng(seed_mode);
    let before_categories = category::Entity::find()
        .order_by_desc(category::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);

    for i in 0..count {
        let name: String = l::Word(EN).fake_with_rng(&mut rng);
        let slug = format!("{}-{}", name.to_lowercase().replace(' ', "-"), rng.random::<u32>());
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

        if let Err(err) = category::Entity::create(db, new_category).await {
            log(format!("Failed to create category {}: {}", name, err));
        } else if (i + 1) % 10 == 0 || i + 1 == count {
            log(format!("Created {} / {} categories", i + 1, count));
        }
    }

    let after_categories = category::Entity::find()
        .order_by_desc(category::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(before_categories);

    Ok(compute_range(before_categories, after_categories))
}

async fn seed_extra_tags<F>(
    db: &DatabaseConnection,
    count: u32,
    seed_mode: Option<SeedMode>,
    log: &F,
) -> SeedResult<TableRange>
where
    F: Fn(String),
{
    let mut rng = seeded_rng(seed_mode);
    let before_tags = tag::Entity::find()
        .order_by_desc(tag::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);

    for i in 0..count {
        let name: String = l::Word(EN).fake_with_rng(&mut rng);
        let slug = format!("{}-{}", name.to_lowercase().replace(' ', "-"), rng.random::<u32>());
        let new_tag = tag::NewTag {
            name: name.clone(),
            slug,
            description: None,
            color: Some("#3b82f6".to_string()),
            text_color: None,
            is_active: Some(true),
        };

        if let Err(err) = tag::Entity::create(db, new_tag).await {
            log(format!("Failed to create tag {}: {}", name, err));
        } else if (i + 1) % 10 == 0 || i + 1 == count {
            log(format!("Created {} / {} tags", i + 1, count));
        }
    }

    let after_tags = tag::Entity::find()
        .order_by_desc(tag::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(before_tags);

    Ok(compute_range(before_tags, after_tags))
}

async fn seed_extra_posts<F>(
    db: &DatabaseConnection,
    count: u32,
    seed_mode: Option<SeedMode>,
    log: &F,
) -> SeedResult<TableRange>
where
    F: Fn(String),
{
    let mut rng = seeded_rng(seed_mode);
    let before_posts = post::Entity::find()
        .order_by_desc(post::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);

    let authors: Vec<user::Model> = user::Entity::find()
        .filter(user::Column::Role.eq(user::UserRole::Author))
        .all(db)
        .await?;
    let categories = category::Entity::find().all(db).await?;
    let tags = tag::Entity::find().all(db).await?;

    if authors.is_empty() || categories.is_empty() || tags.is_empty() {
        return Err(SeedError::Db(
            "Need at least one author, category, and tag to seed posts".to_string(),
        ));
    }

    log(format!(
        "Seeding {} additional posts (preset: {})...",
        count,
        size_label(count)
    ));

    for i in 0..count {
        let author = authors.choose(&mut rng).unwrap();
        let category_id = categories.choose(&mut rng).map(|c| c.id).unwrap();
        let tags_amount = rng.random_range(1..4);
        let tag_ids: Vec<i32> = tags
            .choose_multiple(&mut rng, tags_amount)
            .cloned()
            .map(|t| t.id)
            .collect();
        let post_title: String = l::Sentence(EN, 1..2).fake_with_rng(&mut rng);
        let post_excerpt = l::Words(EN, 1..8)
            .fake_with_rng::<Vec<String>, _>(&mut rng)
            .join(" ");
        let post_content_text: String = l::Paragraph(EN, 1..8).fake_with_rng(&mut rng);
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
            slug: format!(
                "{}-{}",
                post_title.to_lowercase().replace(' ', "-"),
                rng.random::<u32>()
            ),
            content: post_content,
            excerpt: Some(post_excerpt),
            featured_image_id: None,
            status: if is_published {
                post::PostStatus::Published
            } else {
                post::PostStatus::Draft
            },
            author_id: author.id,
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

        let _ = post::Entity::create(db, new_post).await;
        if (i + 1) % 10 == 0 || i + 1 == count {
            log(format!("Created {} / {} posts", i + 1, count));
        }
    }

    let after_posts = post::Entity::find()
        .order_by_desc(post::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(before_posts);

    Ok(compute_range(before_posts, after_posts))
}

async fn seed_extra_comments<F>(
    db: &DatabaseConnection,
    count: u32,
    seed_mode: Option<SeedMode>,
    log: &F,
) -> SeedResult<TableRange>
where
    F: Fn(String),
{
    let mut rng = seeded_rng(seed_mode);
    let before_comments = post_comment::Entity::find()
        .order_by_desc(post_comment::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);

    let posts = post::Entity::find()
        .filter(post::Column::Status.eq(post::PostStatus::Published))
        .all(db)
        .await?;
    let users = user::Entity::find()
        .filter(user::Column::Role.ne(user::UserRole::Admin))
        .all(db)
        .await?;

    if posts.is_empty() || users.is_empty() {
        return Err(SeedError::Db(
            "Need posts and users to seed post comments".to_string(),
        ));
    }

    log(format!(
        "Seeding {} additional post comments (preset: {})...",
        count,
        size_label(count)
    ));

    for i in 0..count {
        let post = posts.choose(&mut rng).unwrap();
        let user = users.choose(&mut rng).unwrap();
        let content: String = l::Sentence(EN, 1..2).fake_with_rng(&mut rng);
        let new_comment = post_comment::NewComment {
            post_id: post.id,
            user_id: user.id,
            content,
            likes_count: Some(0),
        };
        let _ = post_comment::Entity::create(db, new_comment).await;

        if (i + 1) % 20 == 0 || i + 1 == count {
            log(format!("Created {} / {} comments", i + 1, count));
        }
    }

    let after_comments = post_comment::Entity::find()
        .order_by_desc(post_comment::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(before_comments);

    Ok(compute_range(before_comments, after_comments))
}

async fn seed_extra_comment_flags<F>(
    db: &DatabaseConnection,
    count: u32,
    seed_mode: Option<SeedMode>,
    log: &F,
) -> SeedResult<TableRange>
where
    F: Fn(String),
{
    let mut rng = seeded_rng(seed_mode);
    let before_flags = comment_flag::Entity::find()
        .order_by_desc(comment_flag::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);

    let comments = post_comment::Entity::find().all(db).await?;
    let users = user::Entity::find().all(db).await?;
    let reasons = ["spam", "inappropriate", "off-topic", "harassment"];

    if comments.is_empty() || users.is_empty() {
        return Err(SeedError::Db(
            "Need comments and users to seed comment flags".to_string(),
        ));
    }

    log(format!(
        "Seeding {} comment flags (preset: {})...",
        count,
        size_label(count)
    ));

    for i in 0..count {
        let comment = comments.choose(&mut rng).unwrap();
        let flag_user = users.choose(&mut rng).unwrap();
        let reason = reasons.choose(&mut rng).unwrap();

        let active_model = comment_flag::ActiveModel {
            id: ActiveValue::NotSet,
            comment_id: Set(comment.id),
            user_id: Set(flag_user.id),
            reason: Set(Some(reason.to_string())),
            created_at: Set(chrono::Utc::now().fixed_offset()),
        };

        let _ = active_model.insert(db).await?;

        if let Some(existing) = post_comment::Entity::find_by_id(comment.id).one(db).await? {
            let mut comment_model: post_comment::ActiveModel = existing.into();
            comment_model.flags_count = Set(comment.flags_count + 1);
            let _ = comment_model.update(db).await?;
        }

        if (i + 1) % 20 == 0 || i + 1 == count {
            log(format!("Created {} / {} comment flags", i + 1, count));
        }
    }

    let after_flags = comment_flag::Entity::find()
        .order_by_desc(comment_flag::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(before_flags);

    Ok(compute_range(before_flags, after_flags))
}

async fn seed_extra_post_views<F>(
    db: &DatabaseConnection,
    count: u32,
    seed_mode: Option<SeedMode>,
    log: &F,
) -> SeedResult<TableRange>
where
    F: Fn(String),
{
    let mut rng = seeded_rng(seed_mode);
    let before_views = post_view::Entity::find()
        .order_by_desc(post_view::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);

    let posts = post::Entity::find().all(db).await?;
    let users = user::Entity::find().all(db).await?;
    let user_agents = vec![
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64)",
        "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X)",
        "Mozilla/5.0 (Linux; Android 14)",
    ];
    let ips = vec!["203.0.113.10", "198.51.100.42", "10.0.0.24", "172.16.1.15"];

    if posts.is_empty() {
        return Err(SeedError::Db(
            "Need posts to seed post views".to_string(),
        ));
    }

    log(format!(
        "Seeding {} post views (preset: {})...",
        count,
        size_label(count)
    ));

    for i in 0..count {
        let post = posts.choose(&mut rng).unwrap();
        let viewer = users.choose(&mut rng).map(|u| u.id);
        let view = post_view::ActiveModel {
            id: ActiveValue::NotSet,
            post_id: Set(post.id),
            ip_address: Set(Some(ips.choose(&mut rng).unwrap().to_string())),
            user_agent: Set(Some(user_agents.choose(&mut rng).unwrap().to_string())),
            user_id: Set(viewer),
            created_at: Set(chrono::Utc::now().fixed_offset()),
        };
        let _ = view.insert(db).await?;

        let _ = post::Entity::update_many()
            .col_expr(
                post::Column::ViewCount,
                Expr::col(post::Column::ViewCount).add(1),
            )
            .filter(post::Column::Id.eq(post.id))
            .exec(db)
            .await?;

        if (i + 1) % 100 == 0 || i + 1 == count {
            log(format!("Created {} / {} post views", i + 1, count));
        }
    }

    let after_views = post_view::Entity::find()
        .order_by_desc(post_view::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(before_views);

    Ok(compute_range(before_views, after_views))
}

async fn seed_extra_user_sessions<F>(
    db: &DatabaseConnection,
    count: u32,
    seed_mode: Option<SeedMode>,
    log: &F,
) -> SeedResult<TableRange>
where
    F: Fn(String),
{
    let mut rng = seeded_rng(seed_mode);
    let before_sessions = user_session::Entity::find()
        .order_by_desc(user_session::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);

    let users = user::Entity::find().all(db).await?;
    let devices = vec![
        "MacOS · Chrome 126",
        "Windows · Edge 125",
        "iPhone · Safari 17",
        "Android · Chrome 125",
    ];
    let ip_addresses = vec!["192.168.1.100", "10.0.0.50", "172.16.0.25", "203.0.113.1"];

    if users.is_empty() {
        return Err(SeedError::Db(
            "Need users to seed user sessions".to_string(),
        ));
    }

    for i in 0..count {
        let user = users.choose(&mut rng).unwrap();
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

        if (i + 1) % 20 == 0 || i + 1 == count {
            log(format!("Created {} / {} sessions", i + 1, count));
        }
    }

    let after_sessions = user_session::Entity::find()
        .order_by_desc(user_session::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(before_sessions);

    Ok(compute_range(before_sessions, after_sessions))
}

async fn seed_extra_email_verifications<F>(
    db: &DatabaseConnection,
    count: u32,
    seed_mode: Option<SeedMode>,
    log: &F,
) -> SeedResult<TableRange>
where
    F: Fn(String),
{
    let mut rng = seeded_rng(seed_mode);
    let before_verifications = email_verification::Entity::find()
        .order_by_desc(email_verification::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);

    let users = user::Entity::find().all(db).await?;
    if users.is_empty() {
        return Err(SeedError::Db(
            "Need users to seed email verifications".to_string(),
        ));
    }

    for i in 0..count {
        let user = users.choose(&mut rng).unwrap();
        let code = email_verification::Entity::generate_code();
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
            code: Set(verification.code),
            created_at: Set(verification.created_at),
            updated_at: Set(verification.updated_at),
        };

        let _ = active_model.insert(db).await;

        if (i + 1) % 20 == 0 || i + 1 == count {
            log(format!("Created {} / {} email verifications", i + 1, count));
        }
    }

    let after_verifications = email_verification::Entity::find()
        .order_by_desc(email_verification::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(before_verifications);

    Ok(compute_range(before_verifications, after_verifications))
}

async fn seed_extra_forgot_passwords<F>(
    db: &DatabaseConnection,
    count: u32,
    seed_mode: Option<SeedMode>,
    log: &F,
) -> SeedResult<TableRange>
where
    F: Fn(String),
{
    let mut rng = seeded_rng(seed_mode);
    let before_forgot = forgot_password::Entity::find()
        .order_by_desc(forgot_password::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);

    let users = user::Entity::find().all(db).await?;
    if users.is_empty() {
        return Err(SeedError::Db(
            "Need users to seed forgot passwords".to_string(),
        ));
    }

    for i in 0..count {
        let user = users.choose(&mut rng).unwrap();
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
                code: Set(forgot.code),
                created_at: Set(forgot.created_at),
                updated_at: Set(forgot.updated_at),
            };

            let _ = active_model.insert(db).await;

            if (i + 1) % 20 == 0 || i + 1 == count {
                log(format!("Created {} / {} forgot password codes", i + 1, count));
            }
        }
    }

    let after_forgot = forgot_password::Entity::find()
        .order_by_desc(forgot_password::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(before_forgot);

    Ok(compute_range(before_forgot, after_forgot))
}

async fn seed_extra_post_revisions<F>(
    db: &DatabaseConnection,
    count: u32,
    seed_mode: Option<SeedMode>,
    log: &F,
) -> SeedResult<TableRange>
where
    F: Fn(String),
{
    let mut rng = seeded_rng(seed_mode);
    let before_revisions = post_revision::Entity::find()
        .order_by_desc(post_revision::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);

    let posts = post::Entity::find().all(db).await?;
    if posts.is_empty() {
        return Err(SeedError::Db(
            "Need posts to seed post revisions".to_string(),
        ));
    }

    for i in 0..count {
        let post = posts.choose(&mut rng).unwrap();
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

        if (i + 1) % 20 == 0 || i + 1 == count {
            log(format!("Created {} / {} post revisions", i + 1, count));
        }
    }

    let after_revisions = post_revision::Entity::find()
        .order_by_desc(post_revision::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(before_revisions);

    Ok(compute_range(before_revisions, after_revisions))
}

async fn seed_extra_post_series<F>(
    db: &DatabaseConnection,
    count: u32,
    seed_mode: Option<SeedMode>,
    log: &F,
) -> SeedResult<TableRange>
where
    F: Fn(String),
{
    let mut rng = seeded_rng(seed_mode);
    let before_series = post_series::Entity::find()
        .order_by_desc(post_series::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);

    for i in 0..count {
        let words: Vec<String> = l::Words(EN, 1..4).fake_with_rng(&mut rng);
        let title = words.join(" ");
        let description: String = l::Sentence(EN, 4..8).fake_with_rng(&mut rng);

        let series = post_series::Model {
            id: 0,
            name: title.clone(),
            slug: format!("{}-{}", title.to_lowercase().replace(' ', "-"), rng.random::<u32>()),
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

        if (i + 1) % 10 == 0 || i + 1 == count {
            log(format!("Created {} / {} post series", i + 1, count));
        }
    }

    let after_series = post_series::Entity::find()
        .order_by_desc(post_series::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(before_series);

    Ok(compute_range(before_series, after_series))
}

async fn seed_extra_scheduled_posts<F>(
    db: &DatabaseConnection,
    count: u32,
    seed_mode: Option<SeedMode>,
    log: &F,
) -> SeedResult<TableRange>
where
    F: Fn(String),
{
    let mut rng = seeded_rng(seed_mode);
    let before_sched = scheduled_post::Entity::find()
        .order_by_desc(scheduled_post::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);

    let posts = post::Entity::find()
        .filter(post::Column::Status.eq(post::PostStatus::Draft))
        .all(db)
        .await?;
    if posts.is_empty() {
        return Err(SeedError::Db(
            "Need draft posts to seed scheduled posts".to_string(),
        ));
    }

    for i in 0..count {
        let post = posts.choose(&mut rng).unwrap();
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

        if (i + 1) % 10 == 0 || i + 1 == count {
            log(format!("Created {} / {} scheduled posts", i + 1, count));
        }
    }

    let after_sched = scheduled_post::Entity::find()
        .order_by_desc(scheduled_post::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(before_sched);

    Ok(compute_range(before_sched, after_sched))
}

async fn seed_extra_media<F>(
    db: &DatabaseConnection,
    count: u32,
    seed_mode: Option<SeedMode>,
    log: &F,
) -> SeedResult<TableRange>
where
    F: Fn(String),
{
    let mut rng = seeded_rng(seed_mode);
    let before_media = media::Entity::find()
        .order_by_desc(media::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);

    let media_types = vec!["image/png", "image/jpeg", "image/webp"];
    let sizes = vec![1024, 2048, 4096, 8192];

    for i in 0..count {
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

        if (i + 1) % 20 == 0 || i + 1 == count {
            log(format!("Created {} / {} media records", i + 1, count));
        }
    }

    let after_media = media::Entity::find()
        .order_by_desc(media::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(before_media);

    Ok(compute_range(before_media, after_media))
}

async fn seed_extra_media_variants<F>(
    db: &DatabaseConnection,
    count: u32,
    seed_mode: Option<SeedMode>,
    log: &F,
) -> SeedResult<TableRange>
where
    F: Fn(String),
{
    let _ = seed_mode;
    let media_items = media::Entity::find().all(db).await?;
    if media_items.is_empty() {
        return Err(SeedError::Db(
            "Need media to seed media variants".to_string(),
        ));
    }

    let before_variants = media_variant::Entity::find()
        .order_by_desc(media_variant::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);

    let mut created = 0;
    for media_item in media_items.iter().cycle() {
        if created >= count {
            break;
        }
        let variants = vec![
            (format!("{}-thumb", media_item.object_key), 200, 200, 32, "thumbnail"),
            (format!("{}-small", media_item.object_key), 400, 400, 64, "small"),
            (format!("{}-medium", media_item.object_key), 800, 800, 128, "medium"),
        ];

        for (key, w, h, size, variant_type) in variants {
            if created >= count {
                break;
            }
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
            created += 1;
            if created % 20 == 0 || created == count {
                log(format!("Created {} / {} media variants", created, count));
            }
        }
    }

    let after_variants = media_variant::Entity::find()
        .order_by_desc(media_variant::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(before_variants);

    Ok(compute_range(before_variants, after_variants))
}

async fn seed_extra_media_usage<F>(
    db: &DatabaseConnection,
    count: u32,
    seed_mode: Option<SeedMode>,
    log: &F,
) -> SeedResult<TableRange>
where
    F: Fn(String),
{
    let media_items = media::Entity::find().all(db).await?;
    let posts = post::Entity::find().all(db).await?;
    if media_items.is_empty() || posts.is_empty() {
        return Err(SeedError::Db(
            "Need media and posts to seed media usage".to_string(),
        ));
    }

    let before_usage = media_usage::Entity::find()
        .order_by_desc(media_usage::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);

    let mut rng = seeded_rng(seed_mode);
    for i in 0..count {
        let media_item = media_items.choose(&mut rng).unwrap();
        let post = posts.choose(&mut rng).unwrap();
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

        if (i + 1) % 20 == 0 || i + 1 == count {
            log(format!("Created {} / {} media usage records", i + 1, count));
        }
    }

    let after_usage = media_usage::Entity::find()
        .order_by_desc(media_usage::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(before_usage);

    Ok(compute_range(before_usage, after_usage))
}

async fn seed_extra_newsletter_subscribers<F>(
    db: &DatabaseConnection,
    count: u32,
    seed_mode: Option<SeedMode>,
    log: &F,
) -> SeedResult<TableRange>
where
    F: Fn(String),
{
    let mut rng = seeded_rng(seed_mode);
    let before_subs = newsletter_subscriber::Entity::find()
        .order_by_desc(newsletter_subscriber::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);

    let mut emails_set: std::collections::HashSet<String> = std::collections::HashSet::new();

    for i in 0..count {
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

            let _ = active_model.insert(db).await;
        }

        if (i + 1) % 25 == 0 || i + 1 == count {
            log(format!("Created {} / {} newsletter subscribers", i + 1, count));
        }
    }

    let after_subs = newsletter_subscriber::Entity::find()
        .order_by_desc(newsletter_subscriber::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(before_subs);

    Ok(compute_range(before_subs, after_subs))
}

async fn seed_extra_route_status<F>(
    db: &DatabaseConnection,
    count: u32,
    seed_mode: Option<SeedMode>,
    log: &F,
) -> SeedResult<TableRange>
where
    F: Fn(String),
{
    let _ = seed_mode;
    let before_routes = route_status::Entity::find()
        .order_by_desc(route_status::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(0);

    let protected_routes = vec![
        "/admin",
        "/admin/users",
        "/admin/settings",
        "/api/internal",
        "/debug",
        "/health",
        "/metrics",
        "/admin/comments",
    ];

    for i in 0..count {
        let route_pattern = protected_routes
            .get((i as usize) % protected_routes.len())
            .unwrap()
            .to_string();
        let route_status_entry = route_status::Model {
            id: 0,
            route_pattern,
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

        if (i + 1) % 10 == 0 || i + 1 == count {
            log(format!("Created {} / {} route status rows", i + 1, count));
        }
    }

    let after_routes = route_status::Entity::find()
        .order_by_desc(route_status::Column::Id)
        .one(db)
        .await?
        .map(|m| m.id)
        .unwrap_or(before_routes);

    Ok(compute_range(before_routes, after_routes))
}
