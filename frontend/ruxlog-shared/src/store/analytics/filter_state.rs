use chrono::{Duration, Utc};
use dioxus::prelude::*;
use std::sync::OnceLock;

use super::AnalyticsEnvelope;

/// Global filter state for analytics dashboard.
///
/// Provides reactive signals for:
/// - Date range (date_from, date_to)
/// - Period presets (7d, 30d, 90d)
///
/// Updates to these signals should trigger data refetch.
pub struct AnalyticsFilterState {
    /// Start date for analytics queries (YYYY-MM-DD format)
    pub date_from: GlobalSignal<Option<String>>,
    /// End date for analytics queries (YYYY-MM-DD format)
    pub date_to: GlobalSignal<Option<String>>,
    /// Active period preset: "7d", "30d", or "90d"
    pub period_preset: GlobalSignal<Option<String>>,
}

static ANALYTICS_FILTER_STATE: OnceLock<AnalyticsFilterState> = OnceLock::new();

fn default_date_from() -> Option<String> {
    let today = Utc::now().date_naive();
    let seven_days_ago = today - Duration::days(7);
    Some(seven_days_ago.format("%Y-%m-%d").to_string())
}

fn default_date_to() -> Option<String> {
    let today = Utc::now().date_naive();
    Some(today.format("%Y-%m-%d").to_string())
}

fn default_period() -> Option<String> {
    Some("7d".to_string())
}

impl AnalyticsFilterState {
    fn instance() -> &'static Self {
        ANALYTICS_FILTER_STATE.get_or_init(|| Self {
            date_from: GlobalSignal::new(default_date_from),
            date_to: GlobalSignal::new(default_date_to),
            period_preset: GlobalSignal::new(default_period),
        })
    }

    /// Set period preset and update date range accordingly
    pub fn set_period_preset(&self, preset: &str) {
        let today = Utc::now().date_naive();
        let days = match preset {
            "7d" => 7,
            "30d" => 30,
            "90d" => 90,
            _ => 7,
        };

        let start_date = today - Duration::days(days);

        *self.date_from.write() = Some(start_date.format("%Y-%m-%d").to_string());
        *self.date_to.write() = Some(today.format("%Y-%m-%d").to_string());
        *self.period_preset.write() = Some(preset.to_string());
    }

    /// Set custom date range and clear period preset
    pub fn set_date_range(&self, from: Option<String>, to: Option<String>) {
        *self.date_from.write() = from;
        *self.date_to.write() = to;
        *self.period_preset.write() = None; // Clear preset when using custom range
    }

    /// Build an AnalyticsEnvelope from current filter state
    pub fn build_envelope(&self) -> AnalyticsEnvelope {
        // Read current filter state (strings in YYYY-MM-DD format)
        let date_from = self.date_from.read().clone();
        let date_to = self.date_to.read().clone();

        AnalyticsEnvelope {
            date_from,
            date_to,
            page: None,
            per_page: None,
            sort_by: None,
            sort_order: None,
        }
    }

    /// Reset to default 7-day period
    pub fn reset(&self) {
        self.set_period_preset("7d");
    }
}

/// Hook to access global analytics filter state
pub fn use_analytics_filters() -> &'static AnalyticsFilterState {
    AnalyticsFilterState::instance()
}
