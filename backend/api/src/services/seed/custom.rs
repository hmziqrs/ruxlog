use std::collections::HashMap;

use fake::faker::lorem::raw as l;
use fake::locales::EN;
use fake::Fake;
use rand::seq::IndexedRandom;
use rand::Rng;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, Set,
};
use sea_orm::sea_query::Expr;

use crate::db::sea_models::{
    category, comment_flag, post, post_comment, post_view, tag, user,
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

    let count = size.count_for_target(target);
    let range = match target {
        CustomSeedTarget::Posts => seed_extra_posts(db, count, seed_mode, &log).await?,
        CustomSeedTarget::PostComments => seed_extra_comments(db, count, seed_mode, &log).await?,
        CustomSeedTarget::CommentFlags => {
            seed_extra_comment_flags(db, count, seed_mode, &log).await?
        }
        CustomSeedTarget::PostViews => seed_extra_post_views(db, count, seed_mode, &log).await?,
    };

    ranges.insert(
        match target {
            CustomSeedTarget::Posts => "posts",
            CustomSeedTarget::PostComments => "post_comments",
            CustomSeedTarget::CommentFlags => "comment_flags",
            CustomSeedTarget::PostViews => "post_views",
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
