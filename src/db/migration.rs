use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use deadpool_diesel::postgres::Pool;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./src/db/migrations");


pub async fn run_migrations(pool: &Pool) -> Result<(), Box<dyn std::error::Error>> {
    let conn = pool.get().await.unwrap();
    conn.interact(|conn| {
        // conn.
        conn.run_pending_migrations(MIGRATIONS).unwrap();
    }).await.unwrap();

    Ok(())
}
