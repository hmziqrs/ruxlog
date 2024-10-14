use super::errors::DBError;
use deadpool_diesel::{postgres::Pool, InteractError};
use diesel::PgConnection;

pub fn combine_errors<S>(
    nested_result: Result<Result<S, diesel::result::Error>, InteractError>,
) -> Result<S, DBError> {
    let inner_result = nested_result?;
    let value = inner_result?;
    Ok(value)
}

pub async fn execute_db_operation<F, T>(pool: &Pool, operation: F) -> Result<T, DBError>
where
    F: FnOnce(&mut PgConnection) -> Result<T, diesel::result::Error> + Send + 'static,
    T: Send + 'static,
{
    let conn = pool.get().await?;
    let result = conn.interact(operation).await;
    combine_errors(result)
}
