use deadpool_diesel::postgres::{Manager, Pool};
use deadpool_diesel::Runtime;
use diesel::prelude::*;

use std::env;
use std::time::Duration;

use crate::db::utils::execute_db_operation;

// use super::utils::execute_db_operation;

pub async fn get_pool() -> Pool {
    let db_url = env::var("POSTGRE_DB_URL").expect("POSTGRE_DB_URL must be set");

    let manager = Manager::new(db_url, Runtime::Tokio1);
    let pool = Pool::builder(manager)
        .runtime(Runtime::Tokio1)
        .create_timeout(Option::Some(Duration::from_secs(15)))
        .build()
        .expect("Failed to create pool.");

    match execute_db_operation(&pool, move |conn| {
        diesel::sql_query("SELECT 1").execute(conn)
    })
    .await
    {
        Ok(_) => {
            println!("Database connection working");
        }
        Err(e) => {
            panic!("Failed to connect to database: {:?}", e)
        }
    }

    pool
}
