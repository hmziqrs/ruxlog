#![allow(unused)]
#![allow(clippy::all)]


use diesel::prelude::*;
use serde::Serialize;

use crate::db::schema;

#[derive(Queryable, Debug, Selectable, Identifiable, Serialize, PartialEq)]
#[diesel(table_name = schema::users)]

pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}


impl User {
    pub fn find_by_id(conn: &mut PgConnection, id: i32) -> Result<Self, diesel::result::Error> {
        use crate::db::schema::users::dsl::*;
        users.find(id).first(conn)
    }

    pub fn find_all(conn: &mut PgConnection) -> Result<Vec<Self>, diesel::result::Error> {
        use crate::db::schema::users::dsl::*;
        users.load(conn)
    }

    // pub fn create(conn: &PgConnection, name: &str, email: &str) -> Result<Self, diesel::result::Error> {
    //     use crate::schema::users::dsl::*;
    //     let new_user = NewUser {
    //         name: name.to_string(),
    //         email: email.to_string(),
    //     };
    //     diesel::insert_into(users).values(&new_user).get_result(conn)
    // }

    // pub fn update(conn: &PgConnection, id: i32, name: &str, email: &str) -> Result<Self, diesel::result::Error> {
    //     use crate::schema::users::dsl::*;
    //     diesel::update(users.find(id))
    //         .set((name.eq(name), email.eq(email)))
    //         .get_result(conn)
    // }

    // pub fn delete(conn: &PgConnection, id: i32) -> Result<usize, diesel::result::Error> {
    //     use crate::schema::users::dsl::*;
    //     diesel::delete(users.find(id)).execute(conn)
    // }
}