use deadpool_diesel::postgres::Pool as PostgresPool;
use lettre;
use sea_orm::DatabaseConnection;
use tower_sessions_redis_store::fred::prelude::Pool as RedisPool;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PostgresPool,
    pub sea_db: DatabaseConnection,
    pub redis_pool: RedisPool,
    pub mailer: lettre::AsyncSmtpTransport<lettre::Tokio1Executor>,
}
