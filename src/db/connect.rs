use deadpool_diesel::postgres::{Manager, Pool};
use deadpool_diesel::{Runtime, Timeouts};
use diesel::prelude::*;

use std::time::Duration;
use std::{env, time};

use crate::db::errors::DBError;
use crate::db::utils::execute_db_operation;

// use super::utils::execute_db_operation;

pub async fn get_pool() -> Pool {
    let db_url = env::var("POSTGRE_DB_URL").expect("POSTGRE_DB_URL must be set");

    let manager = Manager::new(db_url, Runtime::Tokio1);
    let pool = Pool::builder(manager)
        .runtime(Runtime::Tokio1)
        // .timeouts(Timeouts::wait_millis(5000))
        .create_timeout(Option::Some(Duration::from_secs(15)))
        .build()
        .expect("Failed to create pool.");

    match execute_db_operation(&pool, move |conn| {
        diesel::sql_query("SELECT 1").execute(conn)
    })
    .await
    {
        Ok(_) => {}
        Err(e) => {
            panic!("Failed to connect to database: {:?}", e)
        }
    }

    pool
}
