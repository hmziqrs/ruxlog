use deadpool_diesel::postgres::Pool;
use fred::prelude::RedisPool;
use lettre;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Pool,
    pub redis_pool: RedisPool,
    pub mailer: lettre::AsyncSmtpTransport<lettre::Tokio1Executor>,
}
