use crate::services::supabase::SupabaseClient;
use lettre;
use opentelemetry::metrics::Meter;
use sea_orm::DatabaseConnection;
use tower_sessions_redis_store::fred::prelude::Pool as RedisPool;

#[derive(Clone, Debug)]
pub struct ObjectStorageConfig {
    // S3-compatible storage (Cloudflare R2, Garage, AWS S3, etc.)
    pub region: String,
    pub account_id: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
    pub public_url: String,
    pub endpoint: String,
}

#[derive(Clone, Debug)]
pub struct OptimizerConfig {
    pub enabled: bool,
    pub max_pixels: u64,
    pub keep_original: bool,
    pub default_webp_quality: u8,
}

#[derive(Clone)]
pub struct AppState {
    pub sea_db: DatabaseConnection,
    pub redis_pool: RedisPool,
    pub mailer: lettre::AsyncSmtpTransport<lettre::Tokio1Executor>,
    pub object_storage: ObjectStorageConfig,
    pub s3_client: aws_sdk_s3::Client,
    pub optimizer: OptimizerConfig,
    pub meter: Meter,
    pub supabase: SupabaseClient,
}
