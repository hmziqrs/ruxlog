use deadpool_diesel::postgres::Pool;

pub struct AppState {
    pub db: Pool,
}