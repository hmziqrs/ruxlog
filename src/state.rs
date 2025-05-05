use lettre;
use sea_orm::DatabaseConnection;
use tower_sessions_redis_store::fred::prelude::Pool as RedisPool;

#[derive(Clone)]
pub struct R2Config {
    // R2 configuration
    pub region: String,
    pub account_id: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub public_url: String,
}

#[derive(Clone)]
pub struct AppState {
    pub sea_db: DatabaseConnection,
    pub redis_pool: RedisPool,
    pub mailer: lettre::AsyncSmtpTransport<lettre::Tokio1Executor>,
    pub r2: R2Config,
}
