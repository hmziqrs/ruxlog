use crate::error::{DbResult, ErrorCode, ErrorResponse};
use chrono::Utc;
use sea_orm::{entity::prelude::*, DatabaseTransaction, IntoActiveModel, Order, QueryOrder, Set};

use super::*;

// Admin-specific constants
const ADMIN_PER_PAGE: u64 = 20;

impl Entity {
    // Create a new email verification record
    async fn create<T: ConnectionTrait>(conn: &T, new_verification: NewEmailVerification) -> DbResult<Model> {
        let now = Utc::now().naive_utc();
        let verification = ActiveModel {
            user_id: Set(new_verification.user_id),
            code: Set(new_verification.code),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        match verification.insert(conn).await {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    // Create a new verification record with auto-generated code
    pub async fn create_new<T: ConnectionTrait>(conn: &T, user_id: i32) -> DbResult<Model> {
        let new_verification = NewEmailVerification::new(user_id);
        Self::create(conn, new_verification).await
    }

    // Find email verification record by user_id
    pub async fn find_by_user_id(conn: &DbConn, user_id: i32) -> DbResult<Option<Model>> {
        match Self::find()
            .filter(Column::UserId.eq(user_id))
            .one(conn)
            .await
        {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    // Find email verification record by code
    pub async fn find_by_code(conn: &DbConn, verification_code: &str) -> DbResult<Option<Model>> {
        match Self::find()
            .filter(Column::Code.eq(verification_code))
            .one(conn)
            .await
        {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    // Find email verification record by user_id and code
    pub async fn find_by_user_id_and_code(
        conn: &DbConn,
        user_id: i32,
        code: String,
    ) -> DbResult<Option<Model>> {
        match Self::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::Code.eq(code))
            .one(conn)
            .await
        {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    // Find email verification record by user_id or code
    pub async fn find_by_user_id_or_code(
        conn: &DbConn,
        user_id: Option<i32>,
        code: Option<String>,
    ) -> DbResult<Option<Model>> {
        match (user_id, code) {
            (None, None) => Ok(None),
            (Some(uid), Some(c)) => Self::find_by_user_id_and_code(conn, uid, c).await,
            (Some(uid), None) => Self::find_by_user_id(conn, uid).await,
            (None, Some(c)) => Self::find_by_code(conn, &c).await,
        }
    }

    // Update email verification record
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

    // Regenerate a verification code for a user
    pub async fn regenerate(conn: &DbConn, user_id: i32) -> DbResult<Model> {
        let now = Utc::now().naive_utc();
        let expires_at = now + Self::EXPIRY_TIME;
        let new_code = Self::generate_code();

        let existing = Self::find_by_user_id(conn, user_id).await?;

        if let Some(existing_model) = existing {
            // Update existing verification
            let mut active_model: ActiveModel = existing_model.into_active_model();
            active_model.code = Set(new_code);
            active_model.updated_at = Set(now);

            match active_model.update(conn).await {
                Ok(model) => Ok(model),
                Err(err) => Err(err.into()),
            }
        } else {
            // Create new verification
            let new_verification = NewEmailVerification {
                user_id,
                code: new_code,
            };
            Self::create(conn, new_verification).await
        }
    }

    // Delete email verification record by ID
    pub async fn delete_by_id(conn: &DbConn, verification_id: i32) -> DbResult<u64> {
        match Self::delete_many()
            .filter(Column::Id.eq(verification_id))
            .exec(conn)
            .await
        {
            Ok(result) => Ok(result.rows_affected),
            Err(err) => Err(err.into()),
        }
    }

    // Delete email verification record by user_id
    pub async fn delete_by_user_id(conn: &DbConn, user_id: i32) -> DbResult<u64> {
        match Self::delete_many()
            .filter(Column::UserId.eq(user_id))
            .exec(conn)
            .await
        {
            Ok(result) => Ok(result.rows_affected),
            Err(err) => Err(err.into()),
        }
    }

    // Delete email verification record by code
    pub async fn delete_by_code(conn: &DbConn, verification_code: &str) -> DbResult<u64> {
        match Self::delete_many()
            .filter(Column::Code.eq(verification_code))
            .exec(conn)
            .await
        {
            Ok(result) => Ok(result.rows_affected),
            Err(err) => Err(err.into()),
        }
    }

    // Delete email verification record by user_id and code
    pub async fn delete_by_user_id_and_code(
        conn: &DbConn,
        user_id: i32,
        code: &str,
    ) -> DbResult<u64> {
        match Self::delete_many()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::Code.eq(code))
            .exec(conn)
            .await
        {
            Ok(result) => Ok(result.rows_affected),
            Err(err) => Err(err.into()),
        }
    }

    // Check if a verification code has expired
    pub async fn is_code_expired(conn: &DbConn, user_id: i32, code: &str) -> DbResult<bool> {
        let verification = Self::find_by_user_id_and_code(conn, user_id, code.to_string()).await?;

        match verification {
            Some(model) => {
                Ok(!model.is_expired())
            }
            None => Err(ErrorResponse::new(ErrorCode::RecordNotFound)
                .with_message(&format!("Verification code not found for user {}", user_id))),
        }
    }

    // Check if a verification code is in the delay period
    pub async fn is_in_delay(conn: &DbConn, user_id: i32, code: &str) -> DbResult<bool> {
        let verification = Self::find_by_user_id_and_code(conn, user_id, code.to_string()).await?;

        match verification {
            Some(model) => Ok(model.is_in_delay()),
            None => Err(ErrorResponse::new(ErrorCode::RecordNotFound)
                .with_message(&format!("Verification code not found for user {}", user_id))),
        }
    }

    // Admin query for email verifications with pagination and filtering
    pub async fn admin_query(
        conn: &DbConn,
        query: &AdminEmailVerificationQuery,
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
