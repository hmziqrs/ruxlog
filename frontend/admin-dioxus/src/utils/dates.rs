use chrono::{DateTime, NaiveDateTime, Utc};

/// Format a backend timestamp string (e.g., "2024-05-06T12:34:56.789")
/// into a short human-readable date like "May 6, 2024".
pub fn format_short_date(date_str: &str) -> String {
    if let Ok(date) = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S.%f") {
        date.format("%b %-d, %Y").to_string()
    } else {
        date_str.to_string()
    }
}

/// Format a chrono DateTime<Utc> directly to a short human-readable date like "May 6, 2024".
pub fn format_short_date_dt(dt: &DateTime<Utc>) -> String {
    dt.format("%b %-d, %Y").to_string()
}
