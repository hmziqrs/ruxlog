use std::{collections::BTreeMap, ops::Bound};

use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, TimeZone, Utc};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::{Validate, ValidationError, ValidationErrors};

pub const DEFAULT_PER_PAGE: u64 = 30;
pub const MAX_PER_PAGE: u64 = 200;

/// Shared request envelope for analytics endpoints.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AnalyticsEnvelope {
    #[serde(
        default,
        deserialize_with = "deserialize_optional_date",
        skip_serializing_if = "Option::is_none"
    )]
    pub date_from: Option<NaiveDate>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_date",
        skip_serializing_if = "Option::is_none"
    )]
    pub date_to: Option<NaiveDate>,
    #[serde(default)]
    pub page: Option<u64>,
    #[serde(default)]
    pub per_page: Option<u64>,
    #[serde(default)]
    pub sort_by: Option<String>,
    #[serde(default)]
    pub sort_order: Option<String>,
}

impl AnalyticsEnvelope {
    pub fn resolve(&self) -> ResolvedAnalyticsEnvelope {
        let now = Utc::now().date_naive();

        let upper_bound = self.date_to.unwrap_or(now);
        let lower_bound = self.date_from.unwrap_or_else(|| {
            upper_bound
                .checked_sub_signed(Duration::days(30))
                .unwrap_or(upper_bound)
        });

        let per_page = self
            .per_page
            .map(|value| value.clamp(1, MAX_PER_PAGE))
            .unwrap_or(DEFAULT_PER_PAGE);
        let page = self.page.unwrap_or(1).max(1);

        let sort_order =
            SortOrder::from_option(self.sort_order.as_deref());

        ResolvedAnalyticsEnvelope {
            date_from: start_of_day(lower_bound),
            date_to: end_of_day(upper_bound),
            page,
            per_page,
            sort_by: self
                .sort_by
                .as_ref()
                .map(|value| value.trim().to_lowercase()),
            sort_order,
        }
    }
}

impl Validate for AnalyticsEnvelope {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        if let Some(page) = self.page {
            if page == 0 {
                errors.add(
                    "page",
                    ValidationError::new("min")
                        .with_message("page must be greater than or equal to 1".into()),
                );
            }
        }

        if let Some(per_page) = self.per_page {
            if !(1..=MAX_PER_PAGE).contains(&per_page) {
                errors.add(
                    "per_page",
                    ValidationError::new("range").with_message(
                        format!("per_page must be between 1 and {}", MAX_PER_PAGE).into(),
                    ),
                );
            }
        }

        if let Some(sort_order) = &self.sort_order {
            let normalized = sort_order.trim().to_ascii_lowercase();
            if normalized != "asc" && normalized != "desc" {
                errors.add(
                    "sort_order",
                    ValidationError::new("one_of")
                        .with_message("sort_order must be 'asc' or 'desc'".into()),
                );
            }
        }

        if let (Some(from), Some(to)) = (self.date_from, self.date_to) {
            if from > to {
                errors.add(
                    "date_from",
                    ValidationError::new("lte")
                        .with_message("date_from must be before or equal to date_to".into()),
                );
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedAnalyticsEnvelope {
    pub date_from: DateTimeWithTimeZone,
    pub date_to: DateTimeWithTimeZone,
    pub page: u64,
    pub per_page: u64,
    pub sort_by: Option<String>,
    pub sort_order: SortOrder,
}

impl ResolvedAnalyticsEnvelope {
    pub fn offset(&self) -> u64 {
        (self.page.saturating_sub(1)) * self.per_page
    }

    pub fn bounds(&self) -> (Bound<DateTimeWithTimeZone>, Bound<DateTimeWithTimeZone>) {
        (
            Bound::Included(self.date_from),
            Bound::Included(self.date_to),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl SortOrder {
    pub fn as_sql(&self) -> &'static str {
        match self {
            SortOrder::Asc => "ASC",
            SortOrder::Desc => "DESC",
        }
    }

    fn from_option(value: Option<&str>) -> Self {
        match value {
            Some(v) if v.eq_ignore_ascii_case("asc") => SortOrder::Asc,
            _ => SortOrder::Desc,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AnalyticsInterval {
    Hour,
    #[default]
    Day,
    Week,
    Month,
}

impl AnalyticsInterval {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnalyticsInterval::Hour => "hour",
            AnalyticsInterval::Day => "day",
            AnalyticsInterval::Week => "week",
            AnalyticsInterval::Month => "month",
        }
    }

    pub fn to_bucket_expr(&self, column: &str) -> String {
        match self {
            AnalyticsInterval::Hour => {
                format!("to_char(date_trunc('hour', {column}), 'YYYY-MM-DD HH24:00')")
            }
            AnalyticsInterval::Day => {
                format!("to_char(date_trunc('day', {column}), 'YYYY-MM-DD')")
            }
            AnalyticsInterval::Week => {
                format!("to_char(date_trunc('week', {column}), 'IYYY-\"W\"IW')")
            }
            AnalyticsInterval::Month => {
                format!("to_char(date_trunc('month', {column}), 'YYYY-MM')")
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegistrationTrendsFilters {
    #[serde(default)]
    pub group_by: AnalyticsInterval,
}

impl Default for RegistrationTrendsFilters {
    fn default() -> Self {
        Self {
            group_by: AnalyticsInterval::Day,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationTrendsRequest {
    #[serde(flatten)]
    pub envelope: AnalyticsEnvelope,
    #[serde(default)]
    pub filters: RegistrationTrendsFilters,
}

impl Validate for RegistrationTrendsRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        self.envelope.validate()?;
        self.filters.validate()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RegistrationTrendPoint {
    pub bucket: String,
    pub new_users: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct VerificationRatesFilters {
    #[serde(default)]
    pub group_by: AnalyticsInterval,
}

impl Default for VerificationRatesFilters {
    fn default() -> Self {
        Self {
            group_by: AnalyticsInterval::Day,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRatesRequest {
    #[serde(flatten)]
    pub envelope: AnalyticsEnvelope,
    #[serde(default)]
    pub filters: VerificationRatesFilters,
}

impl Validate for VerificationRatesRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        self.envelope.validate()?;
        self.filters.validate()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct VerificationRatePoint {
    pub bucket: String,
    pub requested: i64,
    pub verified: i64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PublishingTrendsFilters {
    #[serde(default)]
    pub group_by: AnalyticsInterval,
    #[serde(default)]
    pub status: Option<Vec<String>>,
}

impl Default for PublishingTrendsFilters {
    fn default() -> Self {
        Self {
            group_by: AnalyticsInterval::Week,
            status: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishingTrendsRequest {
    #[serde(flatten)]
    pub envelope: AnalyticsEnvelope,
    #[serde(default)]
    pub filters: PublishingTrendsFilters,
}

impl Validate for PublishingTrendsRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        self.envelope.validate()?;
        self.filters.validate()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PublishingTrendPoint {
    pub bucket: String,
    pub counts: BTreeMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PageViewsFilters {
    #[serde(default)]
    pub group_by: AnalyticsInterval,
    #[serde(default)]
    #[validate(range(min = 1))]
    pub post_id: Option<i32>,
    #[serde(default)]
    #[validate(range(min = 1))]
    pub author_id: Option<i32>,
    #[serde(default)]
    pub only_unique: bool,
}

impl Default for PageViewsFilters {
    fn default() -> Self {
        Self {
            group_by: AnalyticsInterval::Day,
            post_id: None,
            author_id: None,
            only_unique: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageViewsRequest {
    #[serde(flatten)]
    pub envelope: AnalyticsEnvelope,
    #[serde(default)]
    pub filters: PageViewsFilters,
}

impl Validate for PageViewsRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        self.envelope.validate()?;
        self.filters.validate()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PageViewPoint {
    pub bucket: String,
    pub views: i64,
    pub unique_visitors: i64,
}

fn default_min_views() -> i64 {
    100
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommentRateSort {
    #[default]
    CommentRate,
    Comments,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CommentRateFilters {
    #[serde(default = "default_min_views")]
    pub min_views: i64,
    #[serde(default)]
    pub sort_by: CommentRateSort,
}

impl Default for CommentRateFilters {
    fn default() -> Self {
        Self {
            min_views: default_min_views(),
            sort_by: CommentRateSort::CommentRate,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentRateRequest {
    #[serde(flatten)]
    pub envelope: AnalyticsEnvelope,
    #[serde(default)]
    pub filters: CommentRateFilters,
}

impl Validate for CommentRateRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        self.envelope.validate()?;
        self.filters.validate()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CommentRatePoint {
    pub post_id: i32,
    pub title: String,
    pub views: i64,
    pub comments: i64,
    pub comment_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct NewsletterGrowthFilters {
    #[serde(default)]
    pub group_by: AnalyticsInterval,
}

impl Default for NewsletterGrowthFilters {
    fn default() -> Self {
        Self {
            group_by: AnalyticsInterval::Week,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsletterGrowthRequest {
    #[serde(flatten)]
    pub envelope: AnalyticsEnvelope,
    #[serde(default)]
    pub filters: NewsletterGrowthFilters,
}

impl Validate for NewsletterGrowthRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        self.envelope.validate()?;
        self.filters.validate()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct NewsletterGrowthPoint {
    pub bucket: String,
    pub new_subscribers: i64,
    pub confirmed: i64,
    pub unsubscribed: i64,
    pub net_growth: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct MediaUploadFilters {
    #[serde(default)]
    pub group_by: AnalyticsInterval,
}

impl Default for MediaUploadFilters {
    fn default() -> Self {
        Self {
            group_by: AnalyticsInterval::Day,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaUploadRequest {
    #[serde(flatten)]
    pub envelope: AnalyticsEnvelope,
    #[serde(default)]
    pub filters: MediaUploadFilters,
}

impl Validate for MediaUploadRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        self.envelope.validate()?;
        self.filters.validate()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MediaUploadPoint {
    pub bucket: String,
    pub upload_count: i64,
    pub total_size_mb: f64,
    pub avg_size_mb: f64,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DashboardPeriod {
    #[serde(rename = "7d")]
    SevenDays,
    #[serde(rename = "30d")]
    #[default]
    ThirtyDays,
    #[serde(rename = "90d")]
    NinetyDays,
}

impl DashboardPeriod {
    pub fn as_str(&self) -> &'static str {
        match self {
            DashboardPeriod::SevenDays => "7d",
            DashboardPeriod::ThirtyDays => "30d",
            DashboardPeriod::NinetyDays => "90d",
        }
    }

    pub fn as_duration(&self) -> Duration {
        match self {
            DashboardPeriod::SevenDays => Duration::days(7),
            DashboardPeriod::ThirtyDays => Duration::days(30),
            DashboardPeriod::NinetyDays => Duration::days(90),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DashboardSummaryFilters {
    #[serde(default)]
    pub period: DashboardPeriod,
}

impl Default for DashboardSummaryFilters {
    fn default() -> Self {
        Self {
            period: DashboardPeriod::ThirtyDays,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummaryRequest {
    #[serde(flatten)]
    pub envelope: Option<AnalyticsEnvelope>,
    #[serde(default)]
    pub filters: DashboardSummaryFilters,
}

impl Validate for DashboardSummaryRequest {
    fn validate(&self) -> Result<(), ValidationErrors> {
        if let Some(envelope) = &self.envelope {
            envelope.validate()?;
        }
        self.filters.validate()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSummaryUsers {
    pub total: i64,
    pub new_in_period: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSummaryPosts {
    pub published: i64,
    pub drafts: i64,
    pub views_in_period: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSummaryEngagement {
    pub comments_in_period: i64,
    pub newsletter_confirmed: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSummaryMedia {
    pub total_files: i64,
    pub uploads_in_period: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DashboardSummaryData {
    pub users: DashboardSummaryUsers,
    pub posts: DashboardSummaryPosts,
    pub engagement: DashboardSummaryEngagement,
    pub media: DashboardSummaryMedia,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsMeta {
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sorted_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters_applied: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl AnalyticsMeta {
    pub fn new(total: u64, page: u64, per_page: u64) -> Self {
        Self {
            total,
            page,
            per_page,
            interval: None,
            sorted_by: None,
            filters_applied: None,
            notes: None,
        }
    }

    pub fn with_interval(mut self, interval: impl Into<String>) -> Self {
        self.interval = Some(interval.into());
        self
    }

    pub fn with_sorted_by(mut self, sorted_by: impl Into<String>) -> Self {
        self.sorted_by = Some(sorted_by.into());
        self
    }

    pub fn with_filters(mut self, filters: Value) -> Self {
        self.filters_applied = Some(filters);
        self
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsEnvelopeResponse<T> {
    pub data: T,
    pub meta: AnalyticsMeta,
}

fn start_of_day(date: NaiveDate) -> DateTimeWithTimeZone {
    let offset = FixedOffset::east_opt(0).expect("UTC offset available");
    offset
        .with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0)
        .single()
        .expect("valid start-of-day datetime")
}

fn end_of_day(date: NaiveDate) -> DateTimeWithTimeZone {
    let offset = FixedOffset::east_opt(0).expect("UTC offset available");
    offset
        .with_ymd_and_hms(date.year(), date.month(), date.day(), 23, 59, 59)
        .single()
        .expect("valid end-of-day datetime")
}

fn deserialize_optional_date<'de, D>(deserializer: D) -> Result<Option<NaiveDate>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Option<String> = Option::deserialize(deserializer)?;
    match value {
        Some(raw) => parse_date(&raw)
            .map(Some)
            .map_err(|err| serde::de::Error::custom(err.to_string())),
        None => Ok(None),
    }
}

fn parse_date(value: &str) -> Result<NaiveDate, chrono::ParseError> {
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return Ok(date);
    }

    DateTime::parse_from_rfc3339(value).map(|dt| dt.date_naive())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    // ── AnalyticsEnvelope validation ─────────────────────────────────────

    #[test]
    fn valid_envelope_with_all_fields_passes() {
        let env = AnalyticsEnvelope {
            date_from: Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
            date_to: Some(NaiveDate::from_ymd_opt(2025, 3, 31).unwrap()),
            page: Some(1),
            per_page: Some(50),
            sort_by: Some("created_at".into()),
            sort_order: Some("asc".into()),
        };
        assert!(env.validate().is_ok());
    }

    #[test]
    fn valid_envelope_with_no_fields_passes() {
        let env = AnalyticsEnvelope {
            date_from: None,
            date_to: None,
            page: None,
            per_page: None,
            sort_by: None,
            sort_order: None,
        };
        assert!(env.validate().is_ok());
    }

    #[test]
    fn page_zero_fails_validation() {
        let env = AnalyticsEnvelope {
            page: Some(0),
            ..Default::default()
        };
        let err = env.validate().unwrap_err();
        assert!(err.field_errors().contains_key("page"));
    }

    #[test]
    fn per_page_zero_fails_validation() {
        let env = AnalyticsEnvelope {
            per_page: Some(0),
            ..Default::default()
        };
        let err = env.validate().unwrap_err();
        assert!(err.field_errors().contains_key("per_page"));
    }

    #[test]
    fn per_page_exceeding_max_fails_validation() {
        let env = AnalyticsEnvelope {
            per_page: Some(MAX_PER_PAGE + 1),
            ..Default::default()
        };
        let err = env.validate().unwrap_err();
        assert!(err.field_errors().contains_key("per_page"));
    }

    #[test]
    fn per_page_at_max_boundary_passes() {
        let env = AnalyticsEnvelope {
            per_page: Some(MAX_PER_PAGE),
            ..Default::default()
        };
        assert!(env.validate().is_ok());
    }

    #[test]
    fn per_page_at_min_boundary_passes() {
        let env = AnalyticsEnvelope {
            per_page: Some(1),
            ..Default::default()
        };
        assert!(env.validate().is_ok());
    }

    #[test]
    fn invalid_sort_order_fails_validation() {
        let env = AnalyticsEnvelope {
            sort_order: Some("invalid".into()),
            ..Default::default()
        };
        let err = env.validate().unwrap_err();
        assert!(err.field_errors().contains_key("sort_order"));
    }

    #[test]
    fn date_from_after_date_to_fails_validation() {
        let env = AnalyticsEnvelope {
            date_from: Some(NaiveDate::from_ymd_opt(2025, 6, 1).unwrap()),
            date_to: Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
            ..Default::default()
        };
        let err = env.validate().unwrap_err();
        assert!(err.field_errors().contains_key("date_from"));
    }

    #[test]
    fn equal_date_from_and_date_to_passes() {
        let d = NaiveDate::from_ymd_opt(2025, 6, 1).unwrap();
        let env = AnalyticsEnvelope {
            date_from: Some(d),
            date_to: Some(d),
            ..Default::default()
        };
        assert!(env.validate().is_ok());
    }

    #[test]
    fn multiple_errors_reported_simultaneously() {
        let env = AnalyticsEnvelope {
            page: Some(0),
            per_page: Some(0),
            sort_order: Some("random".into()),
            date_from: Some(NaiveDate::from_ymd_opt(2025, 12, 1).unwrap()),
            date_to: Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
            ..Default::default()
        };
        let err = env.validate().unwrap_err();
        let fields = err.field_errors();
        assert!(fields.contains_key("page"));
        assert!(fields.contains_key("per_page"));
        assert!(fields.contains_key("sort_order"));
        assert!(fields.contains_key("date_from"));
    }

    #[test]
    fn sort_order_case_insensitive_passes() {
        for order in ["asc", "desc", "ASC", "DESC", "Asc", "Desc"] {
            let env = AnalyticsEnvelope {
                sort_order: Some(order.into()),
                ..Default::default()
            };
            assert!(env.validate().is_ok(), "sort_order '{order}' should be valid");
        }
    }

    // ── AnalyticsEnvelope::resolve() ─────────────────────────────────────

    #[test]
    fn resolve_defaults_to_30_day_window() {
        let env = AnalyticsEnvelope {
            date_from: None,
            date_to: None,
            page: None,
            per_page: None,
            sort_by: None,
            sort_order: None,
        };
        let resolved = env.resolve();

        let expected_duration = resolved.date_to - resolved.date_from;
        // 30 full days = 30 * 86400 seconds; end_of_day adds 23:59:59
        assert!(
            expected_duration.num_days() >= 29 && expected_duration.num_days() <= 30,
            "expected ~30 day window, got {} days",
            expected_duration.num_days()
        );
        assert_eq!(resolved.page, 1);
        assert_eq!(resolved.per_page, DEFAULT_PER_PAGE);
        assert_eq!(resolved.sort_order, SortOrder::Desc);
        assert!(resolved.sort_by.is_none());
    }

    #[test]
    fn resolve_clamps_per_page_to_max() {
        let env = AnalyticsEnvelope {
            per_page: Some(9999),
            ..Default::default()
        };
        let resolved = env.resolve();
        assert_eq!(resolved.per_page, MAX_PER_PAGE);
    }

    #[test]
    fn resolve_clamps_per_page_to_min() {
        let env = AnalyticsEnvelope {
            per_page: Some(0),
            ..Default::default()
        };
        let resolved = env.resolve();
        assert_eq!(resolved.per_page, 1);
    }

    #[test]
    fn resolve_clamps_page_to_min() {
        let env = AnalyticsEnvelope {
            page: Some(0),
            ..Default::default()
        };
        let resolved = env.resolve();
        assert_eq!(resolved.page, 1);
    }

    #[test]
    fn resolve_preserves_custom_dates() {
        let from = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let env = AnalyticsEnvelope {
            date_from: Some(from),
            date_to: Some(to),
            page: None,
            per_page: None,
            sort_by: None,
            sort_order: None,
        };
        let resolved = env.resolve();
        assert_eq!(resolved.date_from.date_naive(), from);
        assert_eq!(resolved.date_to.date_naive(), to);
    }

    #[test]
    fn resolve_date_from_defaults_relative_to_date_to() {
        let to = NaiveDate::from_ymd_opt(2025, 6, 15).unwrap();
        let env = AnalyticsEnvelope {
            date_from: None,
            date_to: Some(to),
            page: None,
            per_page: None,
            sort_by: None,
            sort_order: None,
        };
        let resolved = env.resolve();
        let expected_from = to - Duration::days(30);
        assert_eq!(resolved.date_from.date_naive(), expected_from);
        assert_eq!(resolved.date_to.date_naive(), to);
    }

    #[test]
    fn resolve_sets_start_and_end_of_day() {
        let d = NaiveDate::from_ymd_opt(2025, 3, 15).unwrap();
        let env = AnalyticsEnvelope {
            date_from: Some(d),
            date_to: Some(d),
            page: None,
            per_page: None,
            sort_by: None,
            sort_order: None,
        };
        let resolved = env.resolve();

        // start_of_day: 00:00:00
        assert_eq!(resolved.date_from.time().hour(), 0);
        assert_eq!(resolved.date_from.time().minute(), 0);
        assert_eq!(resolved.date_from.time().second(), 0);

        // end_of_day: 23:59:59
        assert_eq!(resolved.date_to.time().hour(), 23);
        assert_eq!(resolved.date_to.time().minute(), 59);
        assert_eq!(resolved.date_to.time().second(), 59);
    }

    #[test]
    fn resolve_normalizes_sort_by() {
        let env = AnalyticsEnvelope {
            sort_by: Some("  Created_At  ".into()),
            sort_order: Some("ASC".into()),
            ..Default::default()
        };
        let resolved = env.resolve();
        assert_eq!(resolved.sort_by.as_deref(), Some("created_at"));
        assert_eq!(resolved.sort_order, SortOrder::Asc);
    }

    // ── SortOrder::from_option ───────────────────────────────────────────

    #[test]
    fn sort_order_from_option_asc() {
        assert_eq!(SortOrder::from_option(Some("asc")), SortOrder::Asc);
        assert_eq!(SortOrder::from_option(Some("ASC")), SortOrder::Asc);
        assert_eq!(SortOrder::from_option(Some("Asc")), SortOrder::Asc);
    }

    #[test]
    fn sort_order_from_option_desc() {
        assert_eq!(SortOrder::from_option(Some("desc")), SortOrder::Desc);
        assert_eq!(SortOrder::from_option(Some("DESC")), SortOrder::Desc);
    }

    #[test]
    fn sort_order_from_option_none_defaults_to_desc() {
        assert_eq!(SortOrder::from_option(None), SortOrder::Desc);
    }

    #[test]
    fn sort_order_from_option_invalid_defaults_to_desc() {
        assert_eq!(SortOrder::from_option(Some("random")), SortOrder::Desc);
        assert_eq!(SortOrder::from_option(Some("")), SortOrder::Desc);
    }

    #[test]
    fn sort_order_as_sql() {
        assert_eq!(SortOrder::Asc.as_sql(), "ASC");
        assert_eq!(SortOrder::Desc.as_sql(), "DESC");
    }

    // ── AnalyticsInterval::to_bucket_expr ────────────────────────────────

    #[test]
    fn interval_hour_bucket_expr() {
        let expr = AnalyticsInterval::Hour.to_bucket_expr("created_at");
        assert_eq!(
            expr,
            "to_char(date_trunc('hour', created_at), 'YYYY-MM-DD HH24:00')"
        );
    }

    #[test]
    fn interval_day_bucket_expr() {
        let expr = AnalyticsInterval::Day.to_bucket_expr("created_at");
        assert_eq!(
            expr,
            "to_char(date_trunc('day', created_at), 'YYYY-MM-DD')"
        );
    }

    #[test]
    fn interval_week_bucket_expr() {
        let expr = AnalyticsInterval::Week.to_bucket_expr("created_at");
        assert_eq!(
            expr,
            "to_char(date_trunc('week', created_at), 'IYYY-\"W\"IW')"
        );
    }

    #[test]
    fn interval_month_bucket_expr() {
        let expr = AnalyticsInterval::Month.to_bucket_expr("created_at");
        assert_eq!(
            expr,
            "to_char(date_trunc('month', created_at), 'YYYY-MM')"
        );
    }

    #[test]
    fn interval_uses_custom_column_name() {
        let expr = AnalyticsInterval::Day.to_bucket_expr("published_at");
        assert!(expr.contains("published_at"));
    }

    #[test]
    fn interval_as_str() {
        assert_eq!(AnalyticsInterval::Hour.as_str(), "hour");
        assert_eq!(AnalyticsInterval::Day.as_str(), "day");
        assert_eq!(AnalyticsInterval::Week.as_str(), "week");
        assert_eq!(AnalyticsInterval::Month.as_str(), "month");
    }

    #[test]
    fn interval_default_is_day() {
        assert_eq!(AnalyticsInterval::default(), AnalyticsInterval::Day);
    }

    // ── DashboardPeriod ──────────────────────────────────────────────────

    #[test]
    fn dashboard_period_as_str() {
        assert_eq!(DashboardPeriod::SevenDays.as_str(), "7d");
        assert_eq!(DashboardPeriod::ThirtyDays.as_str(), "30d");
        assert_eq!(DashboardPeriod::NinetyDays.as_str(), "90d");
    }

    #[test]
    fn dashboard_period_as_duration() {
        assert_eq!(DashboardPeriod::SevenDays.as_duration(), Duration::days(7));
        assert_eq!(DashboardPeriod::ThirtyDays.as_duration(), Duration::days(30));
        assert_eq!(DashboardPeriod::NinetyDays.as_duration(), Duration::days(90));
    }

    #[test]
    fn dashboard_period_default_is_thirty_days() {
        assert_eq!(DashboardPeriod::default(), DashboardPeriod::ThirtyDays);
    }

    // ── AnalyticsMeta builder ────────────────────────────────────────────

    #[test]
    fn meta_new_sets_required_fields() {
        let meta = AnalyticsMeta::new(100, 2, 25);
        assert_eq!(meta.total, 100);
        assert_eq!(meta.page, 2);
        assert_eq!(meta.per_page, 25);
        assert!(meta.interval.is_none());
        assert!(meta.sorted_by.is_none());
        assert!(meta.filters_applied.is_none());
        assert!(meta.notes.is_none());
    }

    #[test]
    fn meta_with_interval() {
        let meta = AnalyticsMeta::new(100, 1, 30).with_interval("day");
        assert_eq!(meta.interval.as_deref(), Some("day"));
    }

    #[test]
    fn meta_with_sorted_by() {
        let meta = AnalyticsMeta::new(100, 1, 30).with_sorted_by("created_at");
        assert_eq!(meta.sorted_by.as_deref(), Some("created_at"));
    }

    #[test]
    fn meta_with_filters() {
        let filters = serde_json::json!({"status": "published"});
        let meta = AnalyticsMeta::new(100, 1, 30).with_filters(filters.clone());
        assert_eq!(meta.filters_applied, Some(filters));
    }

    #[test]
    fn meta_fluent_chain() {
        let meta = AnalyticsMeta::new(500, 3, 50)
            .with_interval("week")
            .with_sorted_by("views")
            .with_filters(serde_json::json!({"author_id": 42}));

        assert_eq!(meta.total, 500);
        assert_eq!(meta.page, 3);
        assert_eq!(meta.per_page, 50);
        assert_eq!(meta.interval.as_deref(), Some("week"));
        assert_eq!(meta.sorted_by.as_deref(), Some("views"));
        assert!(meta.filters_applied.is_some());
    }

    // ── ResolvedAnalyticsEnvelope::offset() ──────────────────────────────

    #[test]
    fn offset_page_one() {
        let env = AnalyticsEnvelope {
            per_page: Some(25),
            ..Default::default()
        };
        let resolved = env.resolve();
        assert_eq!(resolved.offset(), 0);
    }

    #[test]
    fn offset_page_two() {
        let env = AnalyticsEnvelope {
            page: Some(2),
            per_page: Some(25),
            ..Default::default()
        };
        let resolved = env.resolve();
        assert_eq!(resolved.offset(), 25);
    }

    #[test]
    fn offset_page_five() {
        let env = AnalyticsEnvelope {
            page: Some(5),
            per_page: Some(10),
            ..Default::default()
        };
        let resolved = env.resolve();
        assert_eq!(resolved.offset(), 40);
    }

    // ── ResolvedAnalyticsEnvelope::bounds() ──────────────────────────────

    #[test]
    fn bounds_returns_included_both_ends() {
        let from = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2025, 6, 1).unwrap();
        let env = AnalyticsEnvelope {
            date_from: Some(from),
            date_to: Some(to),
            ..Default::default()
        };
        let resolved = env.resolve();
        let (lo, hi) = resolved.bounds();

        match (lo, hi) {
            (Bound::Included(lo_dt), Bound::Included(hi_dt)) => {
                assert_eq!(lo_dt.date_naive(), from);
                assert_eq!(hi_dt.date_naive(), to);
            }
            _ => panic!("expected Bound::Included for both bounds"),
        }
    }

    // ── Filter defaults ──────────────────────────────────────────────────

    #[test]
    fn registration_trends_filters_default() {
        let filters = RegistrationTrendsFilters::default();
        assert_eq!(filters.group_by, AnalyticsInterval::Day);
        assert!(filters.validate().is_ok());
    }

    #[test]
    fn verification_rates_filters_default() {
        let filters = VerificationRatesFilters::default();
        assert_eq!(filters.group_by, AnalyticsInterval::Day);
        assert!(filters.validate().is_ok());
    }

    #[test]
    fn publishing_trends_filters_default() {
        let filters = PublishingTrendsFilters::default();
        assert_eq!(filters.group_by, AnalyticsInterval::Week);
        assert!(filters.status.is_none());
        assert!(filters.validate().is_ok());
    }

    #[test]
    fn page_views_filters_default() {
        let filters = PageViewsFilters::default();
        assert_eq!(filters.group_by, AnalyticsInterval::Day);
        assert!(filters.post_id.is_none());
        assert!(filters.author_id.is_none());
        assert!(!filters.only_unique);
        assert!(filters.validate().is_ok());
    }

    #[test]
    fn page_views_filters_invalid_post_id() {
        let filters = PageViewsFilters {
            post_id: Some(0),
            ..Default::default()
        };
        assert!(filters.validate().is_err());
    }

    #[test]
    fn page_views_filters_valid_post_id() {
        let filters = PageViewsFilters {
            post_id: Some(42),
            ..Default::default()
        };
        assert!(filters.validate().is_ok());
    }

    #[test]
    fn comment_rate_filters_default() {
        let filters = CommentRateFilters::default();
        assert_eq!(filters.min_views, 100);
        assert_eq!(filters.sort_by, CommentRateSort::CommentRate);
        assert!(filters.validate().is_ok());
    }

    #[test]
    fn comment_rate_sort_default() {
        assert_eq!(CommentRateSort::default(), CommentRateSort::CommentRate);
    }

    #[test]
    fn newsletter_growth_filters_default() {
        let filters = NewsletterGrowthFilters::default();
        assert_eq!(filters.group_by, AnalyticsInterval::Week);
        assert!(filters.validate().is_ok());
    }

    #[test]
    fn media_upload_filters_default() {
        let filters = MediaUploadFilters::default();
        assert_eq!(filters.group_by, AnalyticsInterval::Day);
        assert!(filters.validate().is_ok());
    }

    #[test]
    fn dashboard_summary_filters_default() {
        let filters = DashboardSummaryFilters::default();
        assert_eq!(filters.period, DashboardPeriod::ThirtyDays);
        assert!(filters.validate().is_ok());
    }

    // ── Request validation delegates to envelope ─────────────────────────

    #[test]
    fn registration_trends_request_invalid_envelope_fails() {
        let req = RegistrationTrendsRequest {
            envelope: AnalyticsEnvelope {
                page: Some(0),
                ..Default::default()
            },
            filters: Default::default(),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn verification_rates_request_valid() {
        let req = VerificationRatesRequest {
            envelope: AnalyticsEnvelope {
                date_from: Some(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
                date_to: Some(NaiveDate::from_ymd_opt(2025, 3, 1).unwrap()),
                page: Some(1),
                per_page: Some(10),
                sort_by: None,
                sort_order: None,
            },
            filters: Default::default(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn dashboard_summary_request_with_none_envelope_passes() {
        let req = DashboardSummaryRequest {
            envelope: None,
            filters: Default::default(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn dashboard_summary_request_with_invalid_envelope_fails() {
        let req = DashboardSummaryRequest {
            envelope: Some(AnalyticsEnvelope {
                per_page: Some(999),
                ..Default::default()
            }),
            filters: Default::default(),
        };
        assert!(req.validate().is_err());
    }

    // ── parse_date ───────────────────────────────────────────────────────

    #[test]
    fn parse_date_yyyy_mm_dd() {
        let date = parse_date("2025-06-15").unwrap();
        assert_eq!(
            date,
            NaiveDate::from_ymd_opt(2025, 6, 15).unwrap()
        );
    }

    #[test]
    fn parse_date_rfc3339() {
        let date = parse_date("2025-06-15T12:30:00Z").unwrap();
        assert_eq!(
            date,
            NaiveDate::from_ymd_opt(2025, 6, 15).unwrap()
        );
    }

    #[test]
    fn parse_date_rfc3339_with_offset() {
        let date = parse_date("2025-06-15T12:30:00+05:30").unwrap();
        assert_eq!(
            date,
            NaiveDate::from_ymd_opt(2025, 6, 15).unwrap()
        );
    }

    #[test]
    fn parse_date_invalid_format() {
        assert!(parse_date("not-a-date").is_err());
    }

    #[test]
    fn parse_date_empty_string() {
        assert!(parse_date("").is_err());
    }

    #[test]
    fn parse_date_partial_yyyy_mm() {
        assert!(parse_date("2025-06").is_err());
    }

    #[test]
    fn parse_date_yyyy_mm_dd_preferred_over_rfc3339() {
        // Both formats could parse "2025-06-15" but the YYYY-MM-DD path runs first
        let date = parse_date("2025-06-15").unwrap();
        assert_eq!(
            date,
            NaiveDate::from_ymd_opt(2025, 6, 15).unwrap()
        );
    }

    // ── Constants ────────────────────────────────────────────────────────

    #[test]
    fn constants_are_sensible() {
        assert_eq!(DEFAULT_PER_PAGE, 30);
        assert_eq!(MAX_PER_PAGE, 200);
        assert!(MAX_PER_PAGE > DEFAULT_PER_PAGE);
    }
}
