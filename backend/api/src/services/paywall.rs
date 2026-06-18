//! Server-side paywall enforcement.
//!
//! The single source of truth for "can this viewer read this post's `content`?"
//! Every public read path consults [`user_has_access`] (or the pure
//! [`decide_access`] core) and strips `content` when access is denied, so paid /
//! subscriber-only content is never shipped to an unentitled viewer regardless of
//! what the client claims (plan Phase 4c — the load-bearing fix for the
//! "full paid content served unauthenticated" finding).
//!
//! Entitlement sources:
//! - `Free`           → always granted.
//! - `Paid`           → a `post_purchases` row for `(user_id, post_id)` exists
//!                      (one-time purchase, granted by the verified webhook).
//! - `SubscriberOnly` → the user has an active subscription.

use std::collections::{HashMap, HashSet};

use sea_orm::{ColumnTrait, Condition, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::db::sea_models::{post_access, post_purchase, subscription};
use crate::error::{DbResult, ErrorCode, ErrorResponse};

pub use ruxlog_types::enums::PostAccessType;

/// The access policy for a single post, mirroring a `post_access` row. Posts
/// with no `post_access` row are [`PostAccessType::Free`] (the platform default).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PostAccessPolicy {
    pub access_type: PostAccessType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_cents: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
}

impl PostAccessPolicy {
    /// The implicit policy for a post with no `post_access` row.
    pub fn free() -> Self {
        Self {
            access_type: PostAccessType::Free,
            price_cents: None,
            currency: None,
        }
    }

    /// True when the policy grants content to everyone (no purchase / sub needed).
    pub fn is_open(&self) -> bool {
        matches!(self.access_type, PostAccessType::Free)
    }
}

impl From<post_access::model::Model> for PostAccessPolicy {
    fn from(m: post_access::model::Model) -> Self {
        Self {
            access_type: m.access_type,
            price_cents: m.price_cents,
            currency: m.currency,
        }
    }
}

/// Outcome of an access check: the policy that applied plus whether content was
/// granted. Callers attach the policy to the response so the frontend can render
/// a paywall when `granted == false`.
#[derive(Clone, Debug)]
pub struct AccessOutcome {
    pub policy: PostAccessPolicy,
    pub granted: bool,
}

/// Pure access decision, decoupled from I/O so it is trivially unit-testable.
///
/// `viewer_bypasses` should be true for the post's author or staff roles
/// (Admin/SuperAdmin/Moderator) — they always see content. `has_purchase` and
/// `has_active_subscription` are the entitlement facts loaded from the DB.
pub fn decide_access(
    policy: &PostAccessPolicy,
    viewer_bypasses: bool,
    has_purchase: bool,
    has_active_subscription: bool,
) -> bool {
    if viewer_bypasses || policy.is_open() {
        return true;
    }
    match policy.access_type {
        PostAccessType::Free => true,
        PostAccessType::Paid => has_purchase,
        PostAccessType::SubscriberOnly => has_active_subscription,
    }
}

/// Load the access policy for one post (defaults to Free when no row exists).
pub async fn load_post_access_policy(
    db: &DatabaseConnection,
    post_id: i32,
) -> DbResult<PostAccessPolicy> {
    let row = post_access::Entity::find()
        .filter(post_access::Column::PostId.eq(post_id))
        .one(db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;
    Ok(row.map(PostAccessPolicy::from).unwrap_or_else(PostAccessPolicy::free))
}

/// Batch-load policies for many posts (one query), used by list/feed endpoints.
pub async fn load_post_access_map(
    db: &DatabaseConnection,
    post_ids: &[i32],
) -> DbResult<HashMap<i32, PostAccessPolicy>> {
    let mut map = HashMap::new();
    if post_ids.is_empty() {
        return Ok(map);
    }
    let rows = post_access::Entity::find()
        .filter(post_access::Column::PostId.is_in(post_ids.to_vec()))
        .all(db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;
    for r in rows {
        map.insert(r.post_id, PostAccessPolicy::from(r));
    }
    Ok(map)
}

/// True if `user_id` owns a (permanent) one-time purchase of `post_id`.
pub async fn user_has_post_purchase(
    db: &DatabaseConnection,
    user_id: i32,
    post_id: i32,
) -> DbResult<bool> {
    let count = post_purchase::Entity::find()
        .filter(post_purchase::Column::UserId.eq(user_id))
        .filter(post_purchase::Column::PostId.eq(post_id))
        .count(db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;
    Ok(count > 0)
}

/// Batch variant of [`user_has_post_purchase`]: returns the subset of `post_ids`
/// the user owns. Used by list/feed gating so a page of N posts costs one query,
/// not N.
pub async fn user_purchased_post_ids(
    db: &DatabaseConnection,
    user_id: i32,
    post_ids: &[i32],
) -> DbResult<HashSet<i32>> {
    let mut owned = HashSet::new();
    if post_ids.is_empty() {
        return Ok(owned);
    }
    let rows = post_purchase::Entity::find()
        .filter(post_purchase::Column::UserId.eq(user_id))
        .filter(post_purchase::Column::PostId.is_in(post_ids.to_vec()))
        .all(db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;
    for r in rows {
        owned.insert(r.post_id);
    }
    Ok(owned)
}

/// True if `user_id` has an active subscription: status Active/Trialing and,
/// when a period end is recorded, that end is still in the future.
pub async fn user_has_active_subscription(
    db: &DatabaseConnection,
    user_id: i32,
) -> DbResult<bool> {
    use ruxlog_types::enums::SubscriptionStatus;
    let subs = subscription::Entity::find()
        .filter(subscription::Column::UserId.eq(user_id))
        .filter(
            Condition::any()
                .add(subscription::Column::Status.eq(SubscriptionStatus::Active))
                .add(subscription::Column::Status.eq(SubscriptionStatus::Trialing)),
        )
        .all(db)
        .await
        .map_err(|_| ErrorResponse::new(ErrorCode::QueryError))?;

    let now_ts = chrono::Utc::now().timestamp();
    for s in subs {
        // Fail closed on a missing period end (audit F#5/F#11): an
        // `Active`-status row with NO `current_period_end` should NOT unlock
        // subscriber content indefinitely. Such a row means the provider never
        // told us when the paid period ends — the safe assumption is that the
        // viewer is not currently entitled, rather than granting forever off a
        // stale status. (New subscriptions are created with a period end from
        // the verified webhook; a missing one is the exception, not the rule.)
        let still_in_period = s
            .current_period_end
            .map(|end| end.timestamp() > now_ts)
            .unwrap_or(false);
        if still_in_period {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Full per-post access check for a viewer. `viewer_id` is `None` for anonymous
/// viewers. `viewer_bypasses` short-circuits to granted (author / staff).
pub async fn user_has_access(
    db: &DatabaseConnection,
    viewer_id: Option<i32>,
    post_id: i32,
    viewer_bypasses: bool,
) -> DbResult<AccessOutcome> {
    let policy = load_post_access_policy(db, post_id).await?;

    if viewer_bypasses || policy.is_open() {
        return Ok(AccessOutcome {
            policy,
            granted: true,
        });
    }

    // Anonymous viewers can never satisfy a gated policy.
    let Some(user_id) = viewer_id else {
        return Ok(AccessOutcome {
            policy,
            granted: false,
        });
    };

    let (has_purchase, has_active_sub) = match policy.access_type {
        PostAccessType::Paid => (user_has_post_purchase(db, user_id, post_id).await?, false),
        PostAccessType::SubscriberOnly => (false, user_has_active_subscription(db, user_id).await?),
        // is_open() already handled Free above; keep the arm exhaustive.
        PostAccessType::Free => (false, false),
    };

    let granted = decide_access(&policy, viewer_bypasses, has_purchase, has_active_sub);
    Ok(AccessOutcome { policy, granted })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paid(price: i32) -> PostAccessPolicy {
        PostAccessPolicy {
            access_type: PostAccessType::Paid,
            price_cents: Some(price),
            currency: Some("USD".into()),
        }
    }

    fn sub_only() -> PostAccessPolicy {
        PostAccessPolicy {
            access_type: PostAccessType::SubscriberOnly,
            price_cents: None,
            currency: None,
        }
    }

    #[test]
    fn free_is_always_open_and_granted() {
        let policy = PostAccessPolicy::free();
        assert!(policy.is_open());
        // anonymous, no entitlements
        assert!(decide_access(&policy, false, false, false));
    }

    #[test]
    fn paid_requires_purchase() {
        let policy = paid(499);
        assert!(!policy.is_open());
        // anonymous / no purchase → denied
        assert!(!decide_access(&policy, false, false, false));
        // purchased → granted
        assert!(decide_access(&policy, false, true, false));
        // a subscription alone does NOT unlock a per-post-paid post
        assert!(!decide_access(&policy, false, false, true));
    }

    #[test]
    fn subscriber_only_requires_active_subscription() {
        let policy = sub_only();
        assert!(!policy.is_open());
        assert!(!decide_access(&policy, false, false, false));
        // a one-time purchase does NOT unlock subscriber-only content
        assert!(!decide_access(&policy, false, true, false));
        assert!(decide_access(&policy, false, false, true));
    }

    #[test]
    fn author_and_staff_bypass_the_paywall() {
        // Even a gated post with no entitlements is granted when the viewer is
        // the author or a staff role.
        assert!(decide_access(&paid(499), true, false, false));
        assert!(decide_access(&sub_only(), true, false, false));
    }
}
