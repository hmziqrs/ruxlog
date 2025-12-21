use sea_orm::{entity::prelude::*, Order, PaginatorTrait, QueryFilter, QueryOrder, Set};

use crate::error::DbResult;

use super::*;

/// Actions for the `user_bans` entity
impl Entity {
    pub const PER_PAGE: u64 = 20;

    /// Create a new ban record
    pub async fn create(conn: &DbConn, new_ban: NewUserBan) -> DbResult<Model> {
        let now = chrono::Utc::now().fixed_offset();

        let ban = ActiveModel {
            user_id: Set(new_ban.user_id),
            reason: Set(new_ban.reason),
            banned_by: Set(new_ban.banned_by),
            expires_at: Set(new_ban.expires_at),
            created_at: Set(now),
            revoked_at: Set(None),
            revoked_by: Set(None),
            ..Default::default()
        };

        match ban.insert(conn).await {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    /// Revoke a ban by id
    pub async fn revoke(
        conn: &DbConn,
        ban_id: i32,
        revoked_by: Option<i32>,
    ) -> DbResult<Option<Model>> {
        let existing = match Self::find_by_id(ban_id).one(conn).await {
            Ok(model) => model,
            Err(err) => return Err(err.into()),
        };

        if let Some(model) = existing {
            let now = chrono::Utc::now().fixed_offset();
            let mut active: ActiveModel = model.into();
            active.revoked_at = Set(Some(now));
            active.revoked_by = Set(revoked_by);

            match active.update(conn).await {
                Ok(updated) => Ok(Some(updated)),
                Err(err) => Err(err.into()),
            }
        } else {
            Ok(None)
        }
    }

    /// Check if a user has an active ban
    ///
    /// Returns the active ban if one exists, None otherwise.
    pub async fn get_active_ban(conn: &DbConn, user_id: i32) -> DbResult<Option<Model>> {
        let now = chrono::Utc::now().fixed_offset();

        // Find bans that are:
        // 1. Not revoked (revoked_at IS NULL)
        // 2. Either no expiry (expires_at IS NULL) OR expires_at > now
        let ban = Self::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::RevokedAt.is_null())
            .filter(
                Column::ExpiresAt
                    .is_null()
                    .or(Column::ExpiresAt.gt(now)),
            )
            .order_by(Column::CreatedAt, Order::Desc)
            .one(conn)
            .await;

        match ban {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    /// Check if a user is banned (returns bool for quick checks)
    pub async fn is_banned(conn: &DbConn, user_id: i32) -> DbResult<bool> {
        Ok(Self::get_active_ban(conn, user_id).await?.is_some())
    }

    /// List all bans for a user (paginated)
    pub async fn list_by_user(
        conn: &DbConn,
        user_id: i32,
        page_no: Option<i64>,
    ) -> DbResult<(Vec<Model>, u64)> {
        let page: u64 = match page_no {
            Some(p) if p > 0 => p as u64,
            _ => 1,
        };

        let query = Self::find()
            .filter(Column::UserId.eq(user_id))
            .order_by(Column::CreatedAt, Order::Desc);

        let paginator = query.paginate(conn, Self::PER_PAGE);

        match paginator.num_items().await {
            Ok(total) => match paginator.fetch_page(page - 1).await {
                Ok(results) => Ok((results, total)),
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        }
    }

    /// Admin list with filters (paginated)
    pub async fn admin_list(conn: &DbConn, query: UserBanQuery) -> DbResult<(Vec<Model>, u64)> {
        let mut q = Self::find();

        if let Some(user_id) = query.user_id {
            q = q.filter(Column::UserId.eq(user_id));
        }

        if let Some(active_only) = query.active_only {
            if active_only {
                let now = chrono::Utc::now().fixed_offset();
                q = q.filter(Column::RevokedAt.is_null()).filter(
                    Column::ExpiresAt
                        .is_null()
                        .or(Column::ExpiresAt.gt(now)),
                );
            }
        }

        // Sorting
        let order = if query.sort_order.as_deref() == Some("asc") {
            Order::Asc
        } else {
            Order::Desc
        };

        q = match query.sort_by.as_deref() {
            Some("created_at") => q.order_by(Column::CreatedAt, order),
            Some("expires_at") => q.order_by(Column::ExpiresAt, order),
            Some("user_id") => q.order_by(Column::UserId, order),
            _ => q.order_by(Column::CreatedAt, Order::Desc),
        };

        let page: u64 = match query.page_no {
            Some(p) if p > 0 => p as u64,
            _ => 1,
        };

        let paginator = q.paginate(conn, Self::PER_PAGE);

        match paginator.num_items().await {
            Ok(total) => match paginator.fetch_page(page - 1).await {
                Ok(results) => Ok((results, total)),
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        }
    }
}
