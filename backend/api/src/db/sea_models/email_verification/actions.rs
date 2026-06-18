use crate::error::{DbResult, ErrorCode, ErrorResponse};
use chrono::Utc;
use sea_orm::{entity::prelude::*, Order, QueryOrder, Set};

use super::*;

const ADMIN_PER_PAGE: u64 = 20;

impl Entity {
    /// Create a verification row for a user. The caller generates the plaintext
    /// code, emails it, and passes `hash_code(secret, plaintext)` here — the
    /// plaintext is never stored.
    pub async fn create<T: ConnectionTrait>(
        conn: &T,
        user_id: i32,
        code_hash: String,
    ) -> DbResult<Model> {
        let now = Utc::now().fixed_offset();
        let verification = ActiveModel {
            user_id: Set(user_id),
            code_hash: Set(code_hash),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        match verification.insert(conn).await {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    /// Look up a verification row by user_id and/or the hash of a submitted
    /// code. `code_hash` must already be `hash_code(secret, submitted_code)`.
    pub async fn find_by_user_id_or_code(
        conn: &DbConn,
        user_id: Option<i32>,
        code_hash: Option<String>,
    ) -> DbResult<Model> {
        if user_id.is_none() && code_hash.is_none() {
            return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                .with_message("Either user_id or code must be provided"));
        }

        let mut query = Self::find();

        if let Some(user_id) = user_id {
            query = query.filter(Column::UserId.eq(user_id));
        }
        if let Some(code_hash) = code_hash {
            query = query.filter(Column::CodeHash.eq(code_hash));
        }

        match query.one(conn).await {
            Ok(Some(result)) => Ok(result),
            Ok(None) => Err(ErrorResponse::new(ErrorCode::InvalidInput)
                .with_message("The provided verification code is invalid")),
            Err(err) => Err(err.into()),
        }
    }

    /// Upsert the (already-hashed) code for a user. The caller generates the
    /// plaintext code, emails it, and passes `hash_code(secret, plaintext)` here.
    pub async fn regenerate(conn: &DbConn, user_id: i32, code_hash: String) -> DbResult<Model> {
        let now = Utc::now().fixed_offset();

        let verification = ActiveModel {
            user_id: Set(user_id),
            code_hash: Set(code_hash.clone()),
            updated_at: Set(now),
            ..Default::default()
        };

        let result = Entity::insert(verification)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(Column::UserId)
                    .update_columns([Column::CodeHash, Column::UpdatedAt])
                    .to_owned(),
            )
            .exec_with_returning(conn)
            .await;

        match result {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    /// Delete the verification row for a user. Called after a successful verify
    /// so the (hash of the) code cannot be replayed — single-use. See plan 3d.
    pub async fn consume(conn: &DbConn, user_id: i32) -> DbResult<u64> {
        match Self::delete_many()
            .filter(Column::UserId.eq(user_id))
            .exec(conn)
            .await
        {
            Ok(res) => Ok(res.rows_affected),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn admin_query(
        conn: &DbConn,
        query: &AdminEmailVerificationQuery,
    ) -> DbResult<(Vec<Model>, u64)> {
        let mut db_query = Self::find();

        if let Some(user_id) = query.user_id {
            db_query = db_query.filter(Column::UserId.eq(user_id));
        }

        if let Some(code_hash) = &query.code_hash {
            db_query = db_query.filter(Column::CodeHash.eq(code_hash));
        }

        if let Some(created_at) = query.created_at {
            db_query = db_query.filter(Column::CreatedAt.gte(created_at));
        }

        if let Some(updated_at) = query.updated_at {
            db_query = db_query.filter(Column::UpdatedAt.gte(updated_at));
        }

        if let (Some(sort_by), Some(sort_order)) = (&query.sort_by, &query.sort_order) {
            for field in sort_by {
                let order = if sort_order == "asc" {
                    Order::Asc
                } else {
                    Order::Desc
                };

                match field.as_str() {
                    "id" => db_query = db_query.order_by(Column::Id, order),
                    "user_id" => db_query = db_query.order_by(Column::UserId, order),
                    "code_hash" => db_query = db_query.order_by(Column::CodeHash, order),
                    "created_at" => db_query = db_query.order_by(Column::CreatedAt, order),
                    "updated_at" => db_query = db_query.order_by(Column::UpdatedAt, order),
                    _ => {}
                }
            }
        } else {
            db_query = db_query.order_by(Column::Id, Order::Desc);
        }

        let page = match query.page_no {
            Some(p) if p > 0 => p as u64,
            _ => 1,
        };

        let paginator = db_query.paginate(conn, ADMIN_PER_PAGE);

        match paginator.num_items().await {
            Ok(total) => match paginator.fetch_page(page - 1).await {
                Ok(results) => Ok((results, total)),
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        }
    }
}
