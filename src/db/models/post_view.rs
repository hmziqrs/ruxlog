use chrono::NaiveDateTime;
use deadpool_diesel::postgres::Pool;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::db::{errors::DBError, schema, utils::execute_db_operation};

#[derive(Queryable, Identifiable, Selectable, Debug, PartialEq, Serialize)]
#[diesel(table_name = schema::post_views)]
pub struct PostView {
    pub id: i32,
    pub post_id: i32,
    pub user_id: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = schema::post_views)]
pub struct NewPostView {
    pub post_id: i32,
    pub user_id: Option<i32>,
}

impl PostView {
    pub async fn create(pool: &Pool, new_view: NewPostView) -> Result<Self, DBError> {
        use crate::db::schema::post_views::dsl::*;

        execute_db_operation(pool, move |conn| {
            diesel::insert_into(post_views)
                .values(&new_view)
                .returning(Self::as_returning())
                .get_result(conn)
        })
        .await
    }

    pub fn create_query(
        conn: &mut PgConnection,
        q_post_id: i32,
        q_user_id: Option<i32>,
    ) -> Result<Self, diesel::result::Error> {
        use crate::db::schema::post_views::dsl::*;
        let new_view = NewPostView {
            post_id: q_post_id,
            user_id: q_user_id,
        };

        let view = diesel::insert_into(post_views)
            .values(&new_view)
            .returning(Self::as_returning())
            .get_result(conn)?;

        Ok(view)
    }
}
