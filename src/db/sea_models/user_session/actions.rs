use sea_orm::{entity::prelude::*, Order, PaginatorTrait, QueryFilter, QueryOrder, Set};

use crate::error::DbResult;

use super::*;

/// Actions for the `user_sessions` entity
impl Entity {
    pub const PER_PAGE: u64 = 20;

    /// Create a new session record
    pub async fn create(conn: &DbConn, new_session: NewUserSession) -> DbResult<Model> {
        let now = chrono::Utc::now().fixed_offset();

        let session = ActiveModel {
            user_id: Set(new_session.user_id),
            device: Set(new_session.device),
            ip_address: Set(new_session.ip_address),
            last_seen: Set(now),
            revoked_at: Set(None),
            ..Default::default()
        };

        match session.insert(conn).await {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    /// Update `last_seen` (touch) a session by id
    pub async fn touch(conn: &DbConn, session_id: i32) -> DbResult<Option<Model>> {
        let existing = match Self::find_by_id(session_id).one(conn).await {
            Ok(model) => model,
            Err(err) => return Err(err.into()),
        };

        if let Some(model) = existing {
            let mut active: ActiveModel = model.into();
            active.last_seen = Set(chrono::Utc::now().fixed_offset());

            match active.update(conn).await {
                Ok(updated) => Ok(Some(updated)),
                Err(err) => Err(err.into()),
            }
        } else {
            Ok(None)
        }
    }

    /// Revoke a session by id (sets `revoked_at` and updates `last_seen`)
    pub async fn revoke(conn: &DbConn, session_id: i32) -> DbResult<Option<Model>> {
        let existing = match Self::find_by_id(session_id).one(conn).await {
            Ok(model) => model,
            Err(err) => return Err(err.into()),
        };

        if let Some(model) = existing {
            let now = chrono::Utc::now().fixed_offset();
            let mut active: ActiveModel = model.into();
            active.last_seen = Set(now);
            active.revoked_at = Set(Some(now));

            match active.update(conn).await {
                Ok(updated) => Ok(Some(updated)),
                Err(err) => Err(err.into()),
            }
        } else {
            Ok(None)
        }
    }

    /// List sessions for a specific user (paginated, order by last_seen desc)
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
            .order_by(Column::LastSeen, Order::Desc);

        let paginator = query.paginate(conn, Self::PER_PAGE);

        match paginator.num_items().await {
            Ok(total) => match paginator.fetch_page(page - 1).await {
                Ok(results) => Ok((results, total)),
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        }
    }

    /// Admin list with filters and sorting (paginated)
    pub async fn admin_list(
        conn: &DbConn,
        query: AdminUserSessionQuery,
    ) -> DbResult<(Vec<Model>, u64)> {
        let mut q = Self::find();

        if let Some(user_id) = query.user_id {
            q = q.filter(Column::UserId.eq(user_id));
        }

        if let Some(device_filter) = query.device {
            let pattern = format!("%{}%", device_filter);
            q = q.filter(Column::Device.contains(&pattern));
        }

        if let Some(ip_filter) = query.ip_address {
            let pattern = format!("%{}%", ip_filter);
            q = q.filter(Column::IpAddress.contains(&pattern));
        }

        if let Some(active_only) = query.active_only {
            if active_only {
                q = q.filter(Column::RevokedAt.is_null());
            }
        }

        if let Some(seen_since) = query.seen_since {
            q = q.filter(Column::LastSeen.gte(seen_since));
        }

        if let Some(revoked_since) = query.revoked_since {
            q = q.filter(Column::RevokedAt.gte(revoked_since));
        }

        // Sorting
        if let Some(sort_fields) = &query.sort_by {
            let order = if query.sort_order.as_deref() == Some("asc") {
                Order::Asc
            } else {
                Order::Desc
            };

            for field in sort_fields {
                q = match field.as_str() {
                    "last_seen" => q.order_by(Column::LastSeen, order),
                    "revoked_at" => q.order_by(Column::RevokedAt, order),
                    "user_id" => q.order_by(Column::UserId, order),
                    "id" => q.order_by(Column::Id, order),
                    _ => q,
                };
            }
        } else {
            // Default sort by last_seen desc
            q = q.order_by(Column::LastSeen, Order::Desc);
        }

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
