use lettre;
use sea_orm::DatabaseConnection;
use tower_sessions_redis_store::fred::prelude::Pool as RedisPool;

#[derive(Clone)]
pub struct S3Config {
    // R2 configuration
    pub r2_region: String,
    pub r2_endpoint: String,
    pub r2_bucket: String,
    pub r2_access_key: String,
    pub r2_secret_key: String,
    pub r2_public_url: String,
}

#[derive(Clone)]
pub struct AppState {
    pub sea_db: DatabaseConnection,
    pub redis_pool: RedisPool,
    pub mailer: lettre::AsyncSmtpTransport<lettre::Tokio1Executor>,
    pub s3: S3Config,
}
