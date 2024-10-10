use deadpool_diesel::postgres::{Manager, Pool};
use deadpool_diesel::Runtime;
use std::env;

pub fn get_pool() -> Pool {
    let db_url = env::var("POSTGRES_DB_URL").unwrap();

    let manager = Manager::new(db_url, Runtime::Tokio1);
    let pool = Pool::builder(manager)
        .build()
        .unwrap();
    return pool;
}         