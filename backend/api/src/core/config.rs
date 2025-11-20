use std::env;

#[derive(Clone, Debug)]
pub struct CoreConfig {
    pub postgres_url: String,
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_username: String,
    pub redis_password: String,
}

impl CoreConfig {
    pub fn from_env() -> Self {
        let postgres_user = env::var("POSTGRES_USER").expect("POSTGRES_USER must be set");
        let postgres_password =
            env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD must be set");
        let postgres_db = env::var("POSTGRES_DB").expect("POSTGRES_DB must be set");
        let postgres_host = env::var("POSTGRES_HOST").expect("POSTGRES_HOST must be set");
        let postgres_port = env::var("POSTGRES_PORT").expect("POSTGRES_PORT must be set");

        let postgres_url = format!(
            "postgres://{}:{}@{}:{}/{}",
            postgres_user, postgres_password, postgres_host, postgres_port, postgres_db
        );

        let redis_host = env::var("REDIS_HOST").expect("REDIS_HOST must be set");
        let redis_port = env::var("REDIS_PORT")
            .expect("REDIS_PORT must be set")
            .parse()
            .expect("REDIS_PORT must be a valid u16");
        let redis_username = env::var("REDIS_USER").expect("REDIS_USER must be set");
        let redis_password = env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD must be set");

        Self {
            postgres_url,
            redis_host,
            redis_port,
            redis_username,
            redis_password,
        }
    }
}

