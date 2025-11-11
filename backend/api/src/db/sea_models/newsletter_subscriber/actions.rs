use crate::error::DbResult;
use sea_orm::{entity::prelude::*, Condition, Order, QueryOrder, QuerySelect, Set};

use super::{
    ActiveModel, Column, Entity, Model, NewSubscriber, SubscriberListItem, SubscriberQuery,
    SubscriberStatus, UpdateSubscriber,
};

impl Entity {
    pub const PER_PAGE: u64 = 10;

    pub async fn create(conn: &DbConn, new_subscriber: NewSubscriber) -> DbResult<Model> {
        let now = chrono::Utc::now().fixed_offset();

        // If a subscriber with the same email exists, update token/status instead of inserting
        match Self::find()
            .filter(Column::Email.eq(new_subscriber.email.clone()))
            .one(conn)
            .await
        {
            Ok(Some(existing)) => {
                let mut am: ActiveModel = existing.into();
                am.token = Set(new_subscriber.token);
                am.status = Set(new_subscriber.status);
                am.updated_at = Set(now);
                match am.update(conn).await {
                    Ok(updated) => Ok(updated),
                    Err(err) => Err(err.into()),
                }
            }
            Ok(None) => {
                let model = ActiveModel {
                    email: Set(new_subscriber.email),
                    status: Set(new_subscriber.status),
                    token: Set(new_subscriber.token),
                    created_at: Set(now),
                    updated_at: Set(now),
                    ..Default::default()
                };
                match model.insert(conn).await {
                    Ok(inserted) => Ok(inserted),
                    Err(err) => Err(err.into()),
                }
            }
            Err(err) => Err(err.into()),
        }
    }

    pub async fn confirm(conn: &DbConn, email: &str, token: &str) -> DbResult<Option<Model>> {
        let now = chrono::Utc::now().fixed_offset();

        let sub = Self::find()
            .filter(Condition::all().add(Column::Email.eq(email)))
            .one(conn)
            .await?;

        if let Some(model) = sub {
            // Only confirm if token matches and not already unsubscribed
            if model.token == token && model.status != SubscriberStatus::Unsubscribed {
                let mut am: ActiveModel = model.into();
                am.status = Set(SubscriberStatus::Confirmed);
                am.updated_at = Set(now);
                let updated = am.update(conn).await?;
                Ok(Some(updated))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    pub async fn unsubscribe(
        conn: &DbConn,
        email: &str,
        token: Option<&str>,
    ) -> DbResult<Option<Model>> {
        let now = chrono::Utc::now().fixed_offset();

        let sub = Self::find()
            .filter(Condition::all().add(Column::Email.eq(email)))
            .one(conn)
            .await?;

        if let Some(model) = sub {
            // If a token is provided, require match
            if let Some(t) = token {
                if model.token != t {
                    return Ok(None);
                }
            }
            let mut am: ActiveModel = model.into();
            am.status = Set(SubscriberStatus::Unsubscribed);
            am.updated_at = Set(now);
            let updated = am.update(conn).await?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }

    pub async fn find_with_query(
        conn: &DbConn,
        query: SubscriberQuery,
    ) -> DbResult<(Vec<SubscriberListItem>, u64)> {
        let mut q = Self::find().select_only().columns(vec![
            Column::Id,
            Column::Email,
            Column::Status,
            Column::CreatedAt,
            Column::UpdatedAt,
        ]);

        if let Some(search) = &query.search {
            let pattern = format!("%{}%", search);
            q = q.filter(Column::Email.contains(&pattern));
        }

        if let Some(status) = query.status {
            q = q.filter(Column::Status.eq(status));
        }

        if let Some(sort_fields) = &query.sort_by {
            for field in sort_fields {
                let order = if query.sort_order.as_deref() == Some("asc") {
                    Order::Asc
                } else {
                    Order::Desc
                };
                q = match field.as_str() {
                    "email" => q.order_by(Column::Email, order),
                    "status" => q.order_by(Column::Status, order),
                    "created_at" => q.order_by(Column::CreatedAt, order),
                    "updated_at" => q.order_by(Column::UpdatedAt, order),
                    _ => q,
                };
            }
        } else {
            // Default sorting
            let order = if query.sort_order.as_deref() == Some("asc") {
                Order::Asc
            } else {
                Order::Desc
            };
            q = q.order_by(Column::CreatedAt, order);
        }

        let page = match query.page_no {
            Some(p) if p > 0 => p,
            _ => 1,
        };

        let paginator = q
            .into_model::<SubscriberListItem>()
            .paginate(conn, Self::PER_PAGE);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(page - 1).await?;

        Ok((items, total))
    }

    pub async fn update_by_email(
        conn: &DbConn,
        email: &str,
        update: UpdateSubscriber,
    ) -> DbResult<Option<Model>> {
        let sub = Self::find()
            .filter(Column::Email.eq(email))
            .one(conn)
            .await?;

        if let Some(model) = sub {
            let now = chrono::Utc::now().fixed_offset();
            let mut am: ActiveModel = model.into();

            if let Some(status) = update.status {
                am.status = Set(status);
            }
            if let Some(token) = update.token {
                am.token = Set(token);
            }
            am.updated_at = Set(now);

            let updated = am.update(conn).await?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }
}
