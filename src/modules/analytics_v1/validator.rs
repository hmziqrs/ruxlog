use std::{collections::BTreeMap, ops::Bound};

use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, TimeZone, Utc};
use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::{Validate, ValidationError, ValidationErrors};

pub const DEFAULT_PER_PAGE: u64 = 30;
pub const MAX_PER_PAGE: u64 = 200;

/// Shared request envelope for analytics endpoints.
#[derive(Debug, Clone, Deserialize, Serialize)]
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
            SortOrder::from_option(self.sort_order.as_ref().map(|value| value.as_str()));

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AnalyticsInterval {
    Hour,
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

impl Default for AnalyticsInterval {
    fn default() -> Self {
        AnalyticsInterval::Day
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommentRateSort {
    CommentRate,
    Comments,
}

impl Default for CommentRateSort {
    fn default() -> Self {
        CommentRateSort::CommentRate
    }
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DashboardPeriod {
    #[serde(rename = "7d")]
    SevenDays,
    #[serde(rename = "30d")]
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

impl Default for DashboardPeriod {
    fn default() -> Self {
        DashboardPeriod::ThirtyDays
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
