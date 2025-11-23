use crate::error::{DbResult, ErrorCode, ErrorResponse};
use sea_orm::{entity::prelude::*, Condition, Order, QueryOrder, QuerySelect, Set};
use tracing::{error, info, instrument, warn};

use super::{
    ActiveModel, Column, Entity, Model, NewSubscriber, SubscriberListItem, SubscriberQuery,
    SubscriberStatus, UpdateSubscriber,
};

impl Entity {
    pub const PER_PAGE: u64 = 20;

    #[instrument(skip(conn, new_subscriber), fields(subscriber_id, email = %new_subscriber.email))]
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
                    Ok(updated) => {
                        tracing::Span::current().record("subscriber_id", updated.id);
                        info!(subscriber_id = updated.id, "Newsletter subscriber updated");
                        Ok(updated)
                    }
                    Err(err) => {
                        error!("Failed to update newsletter subscriber: {}", err);
                        Err(err.into())
                    }
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
                    Ok(inserted) => {
                        tracing::Span::current().record("subscriber_id", inserted.id);
                        info!(subscriber_id = inserted.id, "Newsletter subscriber created");
                        Ok(inserted)
                    }
                    Err(err) => {
                        error!("Failed to create newsletter subscriber: {}", err);
                        Err(err.into())
                    }
                }
            }
            Err(err) => {
                error!("Failed to find newsletter subscriber: {}", err);
                Err(err.into())
            }
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

        if let Some(ts) = query.created_at_gt {
            q = q.filter(Column::CreatedAt.gt(ts));
        }
        if let Some(ts) = query.created_at_lt {
            q = q.filter(Column::CreatedAt.lt(ts));
        }
        if let Some(ts) = query.updated_at_gt {
            q = q.filter(Column::UpdatedAt.gt(ts));
        }
        if let Some(ts) = query.updated_at_lt {
            q = q.filter(Column::UpdatedAt.lt(ts));
        }

        if let Some(sorts) = query.sorts {
            for sort in sorts {
                let column = match sort.field.as_str() {
                    "email" => Some(Column::Email),
                    "status" => Some(Column::Status),
                    "created_at" => Some(Column::CreatedAt),
                    "updated_at" => Some(Column::UpdatedAt),
                    _ => None,
                };
                if let Some(col) = column {
                    q = q.order_by(col, sort.order);
                }
            }
        } else {
            q = q.order_by(Column::CreatedAt, Order::Desc);
        }

        let page = match query.page {
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

    pub async fn find_by_id_with_404(conn: &DbConn, subscriber_id: i32) -> DbResult<Model> {
        match Self::find_by_id(subscriber_id).one(conn).await {
            Ok(Some(model)) => Ok(model),
            Ok(None) => Err(ErrorResponse::new(ErrorCode::SubscriberNotFound)
                .with_message(&format!("Subscriber with ID {} not found", subscriber_id))),
            Err(err) => Err(err.into()),
        }
    }

    #[instrument(skip(conn, update), fields(subscriber_id))]
    pub async fn update(
        conn: &DbConn,
        subscriber_id: i32,
        update: UpdateSubscriber,
    ) -> DbResult<Option<Model>> {
        let sub: Option<Model> = Self::find_by_id(subscriber_id).one(conn).await?;

        if let Some(model) = sub {
            let mut am: ActiveModel = model.into();

            if let Some(status) = update.status {
                am.status = Set(status);
            }
            if let Some(token) = update.token {
                am.token = Set(token);
            }
            am.updated_at = Set(update.updated_at);

            match am.update(conn).await {
                Ok(updated) => {
                    info!(subscriber_id, "Newsletter subscriber updated");
                    Ok(Some(updated))
                }
                Err(err) => {
                    error!(subscriber_id, "Failed to update subscriber: {}", err);
                    Err(err.into())
                }
            }
        } else {
            warn!(subscriber_id, "Subscriber not found for update");
            Ok(None)
        }
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
            let mut am: ActiveModel = model.into();

            if let Some(status) = update.status {
                am.status = Set(status);
            }
            if let Some(token) = update.token {
                am.token = Set(token);
            }
            am.updated_at = Set(update.updated_at);

            let updated = am.update(conn).await?;
            Ok(Some(updated))
        } else {
            Ok(None)
        }
    }
}
