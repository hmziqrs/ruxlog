use sea_orm::sea_query::{Expr, SimpleExpr};

pub fn build_public_file_url(public_url: &str, bucket: Option<&str>, object_key: &str) -> String {
    let base = public_url.trim_end_matches('/');
    let key = object_key.trim_start_matches('/');

    match bucket.map(str::trim).filter(|b| !b.is_empty()) {
        Some(bucket) => format!("{}/{}/{}", base, bucket.trim_matches('/'), key),
        None => format!("{}/{}", base, key),
    }
}

fn escape_sql_literal(value: &str) -> String {
    value.replace('\'', "''")
}

/// Builds a SQL expression that computes a public URL from `bucket` + `object_key`, falling back
/// to `object_key` as a full path when `bucket` is NULL/empty (legacy rows).
pub fn public_file_url_expr(public_url: &str, table_alias: &str) -> SimpleExpr {
    // Use a literal here (vs bind params) to avoid driver placeholder differences (e.g. `?` vs `$1`)
    // in custom SQL fragments across backends.
    let base = escape_sql_literal(public_url.trim_end_matches('/'));
    let alias = escape_sql_literal(table_alias);

    Expr::cust(format!(
        "CASE \
            WHEN \"{alias}\".\"bucket\" IS NULL OR \"{alias}\".\"bucket\" = '' \
            THEN CONCAT('{base}', '/', \"{alias}\".\"object_key\") \
            ELSE CONCAT('{base}', '/', \"{alias}\".\"bucket\", '/', \"{alias}\".\"object_key\") \
        END"
    ))
}
