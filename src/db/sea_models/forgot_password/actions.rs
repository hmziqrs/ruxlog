use crate::{
    db::sea_models::user,
    error::{DbResult, ErrorCode, ErrorResponse},
};
use chrono::Utc;
use sea_orm::{
    entity::prelude::*, IntoActiveModel, JoinType, Order, QueryOrder, QuerySelect, Set,
    TransactionTrait,
};

use super::*;

// Admin-specific constants
const ADMIN_PER_PAGE: u64 = 20;

impl Entity {
    // Create a new forgot password record
    pub async fn create(conn: &DbConn, new_forgot_password: NewForgotPassword) -> DbResult<Model> {
        let now = Utc::now().fixed_offset();
        let forgot_password = ActiveModel {
            user_id: Set(new_forgot_password.user_id),
            code: Set(new_forgot_password.code),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        match forgot_password.insert(conn).await {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    // Create a new forgot password record with auto-generated code

    pub async fn find_query(
        conn: &DbConn,
        user_id: Option<i32>,
        email: Option<&str>,
        code: Option<&str>,
    ) -> DbResult<Model> {
        if user_id.is_none() && email.is_none() && code.is_none() {
            return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                .with_message("Either user_id, email or code must be provided"));
        }
        let mut query = Self::find();
        
        if let Some(user_id) = user_id {
            query = query.filter(Column::UserId.eq(user_id));
        }
        if let Some(code) = code {
            query = query.filter(Column::Code.eq(code));
        }
        if let Some(email) = email {
            query = query
                .join(JoinType::InnerJoin, Relation::User.def())
                .filter(user::Column::Email.eq(email));
        }
        match query.one(conn).await {
            Ok(Some(result)) => Ok(result),
            Ok(None) => Err(ErrorResponse::new(ErrorCode::InvalidInput)
                .with_message("The provided verification code is invalid")),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn reset(conn: &DbConn, user_id: i32, password: String) -> DbResult<u64> {
        let trx = conn.begin().await?;

        let delete_query = Self::delete_many()
            .filter(Column::UserId.eq(user_id))
            .exec(&trx)
            .await;

        match delete_query {
            Ok(_) => match user::Entity::change_password(&trx, user_id, password).await {
                Ok(_) => {
                    trx.commit().await?;
                }
                Err(err) => {
                    trx.rollback().await?;
                    return Err(err.into());
                }
            },
            Err(err) => {
                trx.rollback().await?;
                return Err(err.into());
            }
        };
        Ok(1)
    }

    // Regenerate a forgot password code for a user
    pub async fn regenerate(conn: &DbConn, user_id: i32) -> DbResult<Model> {
        let now = Utc::now().fixed_offset();
        let new_code = Self::generate_code();

        let existing = Self::find_query(conn, Some(user_id), None, None).await;

        if let Ok(existing_model) = existing {
            // Update existing forgot password
            let mut active_model: ActiveModel = existing_model.into_active_model();
            active_model.code = Set(new_code);
            active_model.updated_at = Set(now);

            match active_model.update(conn).await {
                Ok(model) => Ok(model),
                Err(err) => Err(err.into()),
            }
        } else {
            // Create new forgot password
            let new_forgot_password = NewForgotPassword {
                user_id,
                code: new_code,
            };
            Self::create(conn, new_forgot_password).await
        }
    }

    // Admin query for forgot passwords with pagination and filtering
    pub async fn admin_query(
        conn: &DbConn,
        query: &AdminForgotPasswordQuery,
    ) -> DbResult<(Vec<Model>, u64)> {
        let mut db_query = Self::find();

        // Apply filters
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

        // Apply sorting
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
            // Default sort by id descending
            db_query = db_query.order_by(Column::Id, Order::Desc);
        }

        // Handle pagination
        let page = match query.page_no {
            Some(p) if p > 0 => p as u64,
            _ => 1,
        };

        let paginator = db_query.paginate(conn, ADMIN_PER_PAGE);

        // Get total count and paginated results
        match paginator.num_items().await {
            Ok(total) => match paginator.fetch_page(page - 1).await {
                Ok(results) => Ok((results, total)),
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        }
    }
}
