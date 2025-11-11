use crate::error::{DbResult, ErrorCode, ErrorResponse};
use chrono::Utc;
use sea_orm::{entity::prelude::*, Order, QueryOrder, Set};

use super::*;

const ADMIN_PER_PAGE: u64 = 20;

impl Entity {
    pub async fn create<T: ConnectionTrait>(conn: &T, user_id: i32) -> DbResult<Model> {
        let code = Self::generate_code();
        let now = Utc::now().fixed_offset();
        let verification = ActiveModel {
            user_id: Set(user_id),
            code: Set(code),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        match verification.insert(conn).await {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn find_by_user_id_or_code(
        conn: &DbConn,
        user_id: Option<i32>,
        code: Option<String>,
    ) -> DbResult<Model> {
        if user_id.is_none() && code.is_none() {
            return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                .with_message("Either user_id or code must be provided"));
        }

        let mut query = Self::find();

        if let Some(user_id) = user_id {
            query = query.filter(Column::UserId.eq(user_id));
        }
        if let Some(code) = code {
            query = query.filter(Column::Code.eq(code));
        }

        match query.one(conn).await {
            Ok(Some(result)) => return Ok(result),
            Ok(None) => {
                return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                    .with_message("The provided verification code is invalid"));
            }
            Err(err) => Err(err.into()),
        }
    }

    pub async fn update(
        conn: &DbConn,
        verification_id: i32,
        update_verification: UpdateEmailVerification,
    ) -> DbResult<Option<Model>> {
        let verification: Option<Model> = match Self::find_by_id(verification_id).one(conn).await {
            Ok(verification) => verification,
            Err(err) => return Err(err.into()),
        };

        if let Some(verification_model) = verification {
            let mut verification_active: ActiveModel = verification_model.into();

            if let Some(code) = update_verification.code {
                verification_active.code = Set(code);
            }

            verification_active.updated_at = Set(update_verification.updated_at);

            match verification_active.update(conn).await {
                Ok(updated_verification) => Ok(Some(updated_verification)),
                Err(err) => Err(err.into()),
            }
        } else {
            Ok(None)
        }
    }

    pub async fn regenerate(conn: &DbConn, user_id: i32) -> DbResult<Model> {
        let now = Utc::now().fixed_offset();
        let new_code = Self::generate_code();

        let verification = ActiveModel {
            user_id: Set(user_id),
            code: Set(new_code.clone()),
            updated_at: Set(now),
            ..Default::default()
        };

        let result = Entity::insert(verification)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(Column::UserId)
                    .update_columns([Column::Code, Column::UpdatedAt])
                    .to_owned(),
            )
            .exec_with_returning(conn)
            .await;

        match result {
            Ok(model) => Ok(model),
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

        if let Some(code) = &query.code {
            db_query = db_query.filter(Column::Code.eq(code));
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
                    "code" => db_query = db_query.order_by(Column::Code, order),
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
