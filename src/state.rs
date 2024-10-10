use deadpool_diesel::postgres::Pool;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Pool,
}