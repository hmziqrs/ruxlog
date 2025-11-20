use std::sync::Arc;

use sea_orm::DatabaseConnection;
use tower_sessions_redis_store::fred::prelude::Pool as RedisPool;

use crate::core::config::CoreConfig;

#[derive(Clone)]
pub struct CoreContext {
    pub config: CoreConfig,
    pub db: DatabaseConnection,
    pub redis: RedisPool,
}

pub type SharedCoreContext = Arc<CoreContext>;

