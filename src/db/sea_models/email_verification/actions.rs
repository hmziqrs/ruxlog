use sea_orm::{entity::prelude::*, Set};
use crate::error::{DbResult, ErrorCode, ErrorResponse};

use super::*;

impl Entity {
    // Create a new email verification record
    pub async fn create(conn: &DbConn, new_verification: NewEmailVerification) -> DbResult<Model> {
        let now = chrono::Utc::now().naive_utc();
        let verification = ActiveModel {
            user_id: Set(new_verification.user_id),
            code: Set(new_verification.code),
            expires_at: Set(new_verification.expires_at),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        match verification.insert(conn).await {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
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

            if let Some(expires_at) = update_verification.expires_at {
                verification_active.expires_at = Set(expires_at);
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

    // Delete email verification record
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

    // Check if a verification code has expired
    pub async fn is_code_expired(conn: &DbConn, user_id: i32, code: &str) -> DbResult<bool> {
        let verification = Self::find_by_user_id_and_code(conn, user_id, code.to_string()).await?;
        
        match verification {
            Some(model) => {
                let now = chrono::Utc::now().naive_utc();
                Ok(model.expires_at < now)
            },
            None => Err(ErrorResponse::new(ErrorCode::RecordNotFound)
                        .with_message(&format!("Verification code not found for user {}", user_id))),
        }
    }
}