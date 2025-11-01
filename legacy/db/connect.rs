use deadpool_diesel::postgres::{Manager, Pool};
use deadpool_diesel::Runtime;
use diesel::prelude::*;

use std::env;

use crate::db::utils::execute_db_operation;

// use super::utils::execute_db_operation;

fn get_db_url() -> String {
    let user = env::var("POSTGRES_USER").expect("POSTGRES_USER must be set");
    let password = env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set");
    let db = env::var("POSTGRES_DB").expect("POSTGRES_DB must be set");
    let host = env::var("POSTGRES_HOST").expect("POSTGRES_HOST must be set");
    let port = env::var("POSTGRES_PORT").expect("POSTGRES_PORT must be set");
    let db_url = format!("postgres://{}:{}@{}:{}/{}", user, password, host, port, db);

    db_url
}

pub async fn get_pool() -> Pool {
    let db_url = get_db_url();

    let manager = Manager::new(db_url, Runtime::Tokio1);
    let pool = Pool::builder(manager)
        .runtime(Runtime::Tokio1)
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
