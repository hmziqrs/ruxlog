use sea_orm::{entity::prelude::*, Set};
use crate::error::DbResult;

use super::*;

impl Entity {
    // Create a new forgot password record
    pub async fn create(conn: &DbConn, new_forgot_password: NewForgotPassword) -> DbResult<Model> {
        let now = chrono::Utc::now().naive_utc();
        let forgot_password = ActiveModel {
            user_id: Set(new_forgot_password.user_id),
            code: Set(new_forgot_password.code),
            expires_at: Set(new_forgot_password.expires_at),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        match forgot_password.insert(conn).await {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    // Find forgot password record by user_id
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

    // Find forgot password record by user_id and code
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

    // Update forgot password record
    pub async fn update(
        conn: &DbConn,
        forgot_password_id: i32,
        update_forgot_password: UpdateForgotPassword,
    ) -> DbResult<Option<Model>> {
        let forgot_password: Option<Model> = match Self::find_by_id(forgot_password_id).one(conn).await {
            Ok(forgot_password) => forgot_password,
            Err(err) => return Err(err.into()),
        };

        if let Some(forgot_password_model) = forgot_password {
            let mut forgot_password_active: ActiveModel = forgot_password_model.into();

            if let Some(code) = update_forgot_password.code {
                forgot_password_active.code = Set(code);
            }

            if let Some(expires_at) = update_forgot_password.expires_at {
                forgot_password_active.expires_at = Set(expires_at);
            }

            forgot_password_active.updated_at = Set(update_forgot_password.updated_at);

            match forgot_password_active.update(conn).await {
                Ok(updated_forgot_password) => Ok(Some(updated_forgot_password)),
                Err(err) => Err(err.into()),
            }
        } else {
            Ok(None)
        }
    }

    // Delete forgot password record
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
}