use std::collections::HashMap;

use fake::{
    faker::internet::en::FreeEmail, faker::lorem::raw as l, faker::name::en::Name, locales::EN,
    Dummy, Fake, Faker,
};
use rand::{rngs::StdRng, seq::IndexedRandom, Rng, SeedableRng};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::db::sea_models::{
    category, comment_flag, email_verification, forgot_password, media, media_usage, media_variant,
    newsletter_subscriber, post, post_comment, post_revision, post_series, post_view, route_status,
    scheduled_post, seed_run, tag, user, user_session,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableRange {
    pub from: i32,
    pub to: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedOutcome {
    pub ranges: HashMap<String, TableRange>,
    pub seed_run_id: Option<i32>,
}

impl SeedOutcome {
    pub fn counts(&self) -> HashMap<String, i32> {
        self.ranges
            .iter()
            .map(|(k, v)| {
                let count = if v.to > 0 && v.to >= v.from {
                    v.to - v.from + 1
                } else {
                    0
                };
                (k.clone(), count)
            })
            .collect()
    }

    pub fn ranges_json(&self) -> Value {
        let mut map = serde_json::Map::new();
        for (k, v) in &self.ranges {
            map.insert(
                k.clone(),
                json!({
                    "from": v.from,
                    "to": v.to,
                }),
            );
        }
        Value::Object(map)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SeedError {
    #[error("database error: {0}")]
    Db(String),
}

impl From<sea_orm::DbErr> for SeedError {
    fn from(value: sea_orm::DbErr) -> Self {
        SeedError::Db(value.to_string())
    }
}

type SeedResult<T> = Result<T, SeedError>;

#[derive(Debug, Dummy)]
struct FakeWord(#[dummy(faker = "l::Word(EN)")] String);

#[derive(Debug, Dummy)]
struct FakeUser {
    #[dummy(faker = "Name()")]
    name: String,
    #[dummy(faker = "FreeEmail()")]
    email: String,
}

fn compute_range(before: i32, after: i32) -> TableRange {
    if after > before {
        TableRange {
            from: before + 1,
            to: after,
        }
    } else {
        TableRange { from: 0, to: 0 }
    }
}

/// Seed everything locally (no Supabase) and record ranges into `seed_runs`.
pub async fn seed_all(db: &DatabaseConnection) -> SeedResult<SeedOutcome> {
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

        match category::Entity::create(db, new_category).await {
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

        match tag::Entity::create(db, new_tag).await {
            Ok(tag) => tags.push(tag),
            Err(err) => {
                println!("Error creating tag: {:?}", err);
            }
        }
    }

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
                    }
                    Err(err) => {
                        println!("Error creating post: {:?}", err);
                    }
                }
            }
        }
    }

    for user in fake_users.iter() {
        if user.role == user::UserRole::User && !fake_posts.is_empty() {
            let num_comments = rng.random_range(1..4);
            for _ in 0..num_comments {
                let post = fake_posts.choose(&mut rng).unwrap();
                let content: String = l::Sentence(EN, 1..2).fake();
                let new_comment = post_comment::NewComment {
                    post_id: post.id,
                    user_id: user.id,
                    content,
                    likes_count: Some(0),
                };

                if let Err(err) = post_comment::Entity::create(db, new_comment).await {
                    println!("Error creating comment: {:?}", err);
                }
            }
        }
    }

    // Seed additional models similar to original handler
    seed_user_sessions(db).await.ok();
    seed_email_verifications(db).await.ok();
    seed_forgot_passwords(db).await.ok();
    seed_post_revisions(db).await.ok();
    seed_post_series(db).await.ok();
    seed_post_views(db).await.ok();
    seed_scheduled_posts(db).await.ok();
    seed_media(db).await.ok();
    seed_media_variants(db).await.ok();
    seed_media_usage(db).await.ok();
    seed_comment_flags(db).await.ok();
    seed_newsletter_subscribers(db).await.ok();
    seed_route_status(db).await.ok();

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
    ranges.insert("users".to_string(), compute_range(before_users, after_users));
    ranges.insert(
        "categories".to_string(),
        compute_range(before_categories, after_categories),
    );
    ranges.insert("tags".to_string(), compute_range(before_tags, after_tags));
    ranges.insert("posts".to_string(), compute_range(before_posts, after_posts));
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

    let seed_run_record = seed_run::ActiveModel {
        key: Set("seed".to_string()),
        ranges: Set(SeedOutcome { ranges: ranges.clone(), seed_run_id: None }.ranges_json()),
        ..Default::default()
    };
    let inserted = seed_run_record.insert(db).await.ok();
    let seed_run_id = inserted.map(|m| m.id);

    Ok(SeedOutcome {
        ranges,
        seed_run_id,
    })
}

pub async fn list_seed_runs(db: &DatabaseConnection) -> SeedResult<Vec<SeedOutcomeRow>> {
    let runs = seed_run::Entity::find()
        .order_by_desc(seed_run::Column::RanAt)
        .all(db)
        .await?;

    let mut rows = Vec::new();
    for r in runs {
        let ranges_map: HashMap<String, TableRange> = serde_json::from_value(r.ranges.clone())
            .unwrap_or_default();
        let counts = ranges_map
            .iter()
            .map(|(k, v)| {
                let count = if v.to > 0 && v.to >= v.from {
                    v.to - v.from + 1
                } else {
                    0
                };
                (k.clone(), count)
            })
            .collect();
        rows.push(SeedOutcomeRow {
            id: r.id,
            key: r.key,
            ran_at: r.ran_at,
            ranges: ranges_map,
            counts,
        });
    }
    Ok(rows)
}

#[derive(Debug, Clone)]
pub struct SeedOutcomeRow {
    pub id: i32,
    pub key: String,
    pub ran_at: chrono::DateTime<chrono::FixedOffset>,
    pub ranges: HashMap<String, TableRange>,
    pub counts: HashMap<String, i32>,
}

#[derive(Debug, Clone)]
pub struct UndoOutcome {
    pub deleted: HashMap<String, u64>,
}

/// Undo a specific seed run based on ID ranges.
pub async fn undo_seed_run(db: &DatabaseConnection, run_id: i32) -> SeedResult<UndoOutcome> {
    let run = seed_run::Entity::find_by_id(run_id)
        .one(db)
        .await?
        .ok_or_else(|| SeedError::Db("Seed run not found".to_string()))?;

    let ranges: HashMap<String, TableRange> =
        serde_json::from_value(run.ranges).unwrap_or_default();

    let mut deleted: HashMap<String, u64> = HashMap::new();

    macro_rules! del_range {
        ($name:literal, $entity:path, $column:expr) => {
            if let Some(range) = ranges.get($name) {
                if range.to > 0 && range.to >= range.from {
                    let res = <$entity as sea_orm::EntityTrait>::delete_many()
                        .filter($column.gte(range.from))
                        .filter($column.lte(range.to))
                        .exec(db)
                        .await?;
                    deleted.insert($name.to_string(), res.rows_affected);
                }
            }
        };
    }

    // Dependency-aware order
    del_range!("comment_flags", comment_flag::Entity, comment_flag::Column::Id);
    del_range!("post_views", post_view::Entity, post_view::Column::Id);
    del_range!(
        "post_comments",
        post_comment::Entity,
        post_comment::Column::Id
    );
    del_range!(
        "post_revisions",
        post_revision::Entity,
        post_revision::Column::Id
    );
    // post_series_posts not tracked; skip.
    del_range!("post_series", post_series::Entity, post_series::Column::Id);
    del_range!(
        "scheduled_posts",
        scheduled_post::Entity,
        scheduled_post::Column::Id
    );
    del_range!("media_usage", media_usage::Entity, media_usage::Column::Id);
    del_range!("media_variants", media_variant::Entity, media_variant::Column::Id);
    del_range!("media", media::Entity, media::Column::Id);
    del_range!("posts", post::Entity, post::Column::Id);
    del_range!("tags", tag::Entity, tag::Column::Id);
    del_range!("categories", category::Entity, category::Column::Id);
    del_range!(
        "user_sessions",
        user_session::Entity,
        user_session::Column::Id
    );
    del_range!(
        "email_verifications",
        email_verification::Entity,
        email_verification::Column::Id
    );
    del_range!(
        "forgot_passwords",
        forgot_password::Entity,
        forgot_password::Column::Id
    );
    del_range!(
        "newsletter_subscribers",
        newsletter_subscriber::Entity,
        newsletter_subscriber::Column::Id
    );
    del_range!(
        "route_status",
        route_status::Entity,
        route_status::Column::Id
    );
    del_range!("users", user::Entity, user::Column::Id);

    Ok(UndoOutcome { deleted })
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
    let mut rng = StdRng::seed_from_u64(999);

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

    for user in users {
        let verification = email_verification::Model {
            id: 0,
            user_id: user.id,
            code: email_verification::Entity::generate_code(),
            created_at: chrono::Utc::now().fixed_offset(),
            updated_at: chrono::Utc::now().fixed_offset(),
        };

        let active_model = email_verification::ActiveModel {
            id: ActiveValue::NotSet,
            user_id: Set(verification.user_id),
            code: Set(verification.code),
            created_at: Set(verification.created_at),
            updated_at: Set(verification.updated_at),
        };

        let _ = active_model.insert(db).await;
    }
    Ok(())
}

async fn seed_forgot_passwords(db: &DatabaseConnection) -> SeedResult<()> {
    let users = user::Entity::find().all(db).await?;
    let mut rng = StdRng::seed_from_u64(5678);

    for user in users.into_iter().take(10) {
        let forgot_password = forgot_password::Model {
            id: 0,
            user_id: user.id,
            code: forgot_password::Entity::generate_code(),
            created_at: chrono::Utc::now().fixed_offset(),
            updated_at: chrono::Utc::now().fixed_offset(),
        };

        let active_model = forgot_password::ActiveModel {
            id: ActiveValue::NotSet,
            user_id: Set(forgot_password.user_id),
            code: Set(forgot_password.code),
            created_at: Set(forgot_password.created_at),
            updated_at: Set(forgot_password.updated_at),
        };

        let _ = active_model.insert(db).await;
    }
    Ok(())
}

async fn seed_post_revisions(db: &DatabaseConnection) -> SeedResult<()> {
    let posts = post::Entity::find().all(db).await?;
    let mut rng = StdRng::seed_from_u64(3456);

    for post in posts.into_iter().take(30) {
        let revision_count = rng.random_range(1..5);
        for i in 0..revision_count {
            let content_text = l::Paragraphs(EN, 1..3)
                .fake::<Vec<String>>()
                .join(" ");
            let post_content = serde_json::json!({
                "time": chrono::Utc::now().timestamp_millis(),
                "blocks": [
                    {"type": "paragraph", "data": {"text": content_text}}
                ],
                "version": "2.30.7"
            });

            let revision = post_revision::Model {
                id: 0,
                post_id: post.id,
                content: post_content.to_string(),
                metadata: Some(serde_json::json!({
                    "title": format!("{} (Revision {})", post.title, i + 1)
                })),
                created_at: chrono::Utc::now().fixed_offset()
                    - chrono::Duration::hours(i as i64 * 24),
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
    let series_names = vec![
        "Getting Started with Rust",
        "Web Development Best Practices",
        "Database Design Patterns",
        "API Security Guide",
        "Frontend Frameworks Comparison",
    ];

    for name in series_names.iter() {
        let new_series = post_series::Model {
            id: 0,
            name: name.to_string(),
            slug: name.to_lowercase().replace(' ', "-"),
            description: Some(format!("A comprehensive series about {}", name)),
            created_at: chrono::Utc::now().fixed_offset(),
            updated_at: chrono::Utc::now().fixed_offset(),
        };

        let active_model = post_series::ActiveModel {
            id: ActiveValue::NotSet,
            name: Set(new_series.name),
            slug: Set(new_series.slug),
            description: Set(new_series.description),
            created_at: Set(new_series.created_at),
            updated_at: Set(new_series.updated_at),
        };

        let _ = active_model.insert(db).await;
    }
    Ok(())
}

async fn seed_post_views(db: &DatabaseConnection) -> SeedResult<()> {
    let posts = post::Entity::find().all(db).await?;
    let users = user::Entity::find().all(db).await?;
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
                id: 0,
                post_id: post.id,
                user_id,
                ip_address: Some(ip_addresses.choose(&mut rng).unwrap().to_string()),
                user_agent: Some("Mozilla/5.0 (compatible; RuxlogBot/1.0)".to_string()),
                created_at: chrono::Utc::now().fixed_offset()
                    - chrono::Duration::minutes(rng.random_range(1..4320)),
            };

            let active_model = post_view::ActiveModel {
                id: ActiveValue::NotSet,
                post_id: Set(view.post_id),
                user_id: Set(view.user_id),
                ip_address: Set(view.ip_address),
                user_agent: Set(view.user_agent),
                created_at: Set(view.created_at),
            };

            let _ = active_model.insert(db).await;
        }
    }
    Ok(())
}

async fn seed_scheduled_posts(db: &DatabaseConnection) -> SeedResult<()> {
    let posts = post::Entity::find().all(db).await?;
    let mut rng = StdRng::seed_from_u64(9999);

    for post in posts.into_iter().take(10) {
        let scheduled_post = scheduled_post::Model {
            id: 0,
            post_id: post.id,
            publish_at: chrono::Utc::now().fixed_offset()
                + chrono::Duration::days(rng.random_range(1..30)),
            status: scheduled_post::ScheduledPostStatus::Pending,
            created_at: chrono::Utc::now().fixed_offset(),
            updated_at: chrono::Utc::now().fixed_offset(),
        };

        let active_model = scheduled_post::ActiveModel {
            id: ActiveValue::NotSet,
            post_id: Set(scheduled_post.post_id),
            publish_at: Set(scheduled_post.publish_at),
            status: Set(scheduled_post.status),
            created_at: Set(scheduled_post.created_at),
            updated_at: Set(scheduled_post.updated_at),
        };

        let _ = active_model.insert(db).await;
    }
    Ok(())
}

async fn seed_media(db: &DatabaseConnection) -> SeedResult<()> {
    let mut rng = StdRng::seed_from_u64(5555);
    for i in 0..20 {
        let media_item = media::Model {
            id: 0,
            object_key: format!("media_{i}.jpg"),
            file_url: format!("https://example.com/media_{i}.jpg"),
            mime_type: "image/jpeg".to_string(),
            width: Some(800),
            height: Some(600),
            size: 1024 * 100,
            extension: Some("jpg".to_string()),
            uploader_id: None,
            reference_type: None,
            content_hash: None,
            is_optimized: rng.random_bool(0.5),
            optimized_at: None,
            created_at: chrono::Utc::now().fixed_offset(),
            updated_at: chrono::Utc::now().fixed_offset(),
        };

        let active_model = media::ActiveModel {
            id: ActiveValue::NotSet,
            object_key: Set(media_item.object_key),
            file_url: Set(media_item.file_url),
            mime_type: Set(media_item.mime_type),
            width: Set(media_item.width),
            height: Set(media_item.height),
            size: Set(media_item.size),
            extension: Set(media_item.extension),
            uploader_id: Set(media_item.uploader_id),
            reference_type: Set(media_item.reference_type),
            content_hash: Set(media_item.content_hash),
            is_optimized: Set(media_item.is_optimized),
            optimized_at: Set(media_item.optimized_at),
            created_at: Set(media_item.created_at),
            updated_at: Set(media_item.updated_at),
        };

        let _ = active_model.insert(db).await;
    }
    Ok(())
}

async fn seed_media_variants(db: &DatabaseConnection) -> SeedResult<()> {
    let media_items = media::Entity::find().all(db).await?;
    let mut rng = StdRng::seed_from_u64(8888);

    for media_item in media_items.into_iter().take(10) {
        let variant = media_variant::Model {
            id: 0,
            media_id: media_item.id,
            object_key: format!("variant_{}_webp", media_item.id),
            mime_type: "image/webp".to_string(),
            width: Some(600),
            height: Some(400),
            size: 50_000,
            extension: Some("webp".to_string()),
            quality: Some(80),
            variant_type: "webp".to_string(),
            created_at: chrono::Utc::now().fixed_offset(),
            updated_at: chrono::Utc::now().fixed_offset(),
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

    Ok(())
}

async fn seed_media_usage(db: &DatabaseConnection) -> SeedResult<()> {
    let media_items = media::Entity::find().all(db).await?;
    let posts = post::Entity::find().all(db).await?;
    let mut rng = StdRng::seed_from_u64(9999);

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

    let mut rng = StdRng::seed_from_u64(1111);
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
    let mut rng = StdRng::seed_from_u64(2222);

    for _ in 0..100 {
        let email = FreeEmail().fake::<String>();
        if emails_set.insert(email.clone()) {
            let status = if rng.random_bool(0.85) {
                newsletter_subscriber::SubscriberStatus::Confirmed
            } else if rng.random_bool(0.1) {
                newsletter_subscriber::SubscriberStatus::Pending
            } else {
                newsletter_subscriber::SubscriberStatus::Unsubscribed
            };

            let subscriber = newsletter_subscriber::Model {
                id: 0,
                email,
                status,
                token: format!("token_{}", rng.random_range(1000..9999)),
                created_at: chrono::Utc::now().fixed_offset(),
                updated_at: chrono::Utc::now().fixed_offset(),
            };

            let active_model = newsletter_subscriber::ActiveModel {
                id: ActiveValue::NotSet,
                email: Set(subscriber.email),
                status: Set(subscriber.status),
                token: Set(subscriber.token),
                created_at: Set(subscriber.created_at),
                updated_at: Set(subscriber.updated_at),
            };

            match active_model.insert(db).await {
                Ok(sub) => subscribers.push(sub),
                Err(err) => println!("Error creating newsletter subscriber: {:?}", err),
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
