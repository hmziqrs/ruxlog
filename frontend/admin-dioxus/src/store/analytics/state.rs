use crate::store::StateFrame;
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

// ========== Shared Types ==========

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AnalyticsInterval {
    Hour,
    Day,
    Week,
    Month,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AnalyticsEnvelope {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyticsMeta {
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sorted_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters_applied: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnalyticsEnvelopeResponse<T> {
    pub data: T,
    pub meta: AnalyticsMeta,
}

// ========== Registration Trends ==========

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegistrationTrendsFilters {
    pub group_by: AnalyticsInterval,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegistrationTrendsRequest {
    pub envelope: AnalyticsEnvelope,
    pub filters: RegistrationTrendsFilters,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegistrationTrendPoint {
    pub bucket: String,
    pub new_users: i64,
}

// ========== Verification Rates ==========

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerificationRatesFilters {
    pub group_by: AnalyticsInterval,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerificationRatesRequest {
    pub envelope: AnalyticsEnvelope,
    pub filters: VerificationRatesFilters,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VerificationRatePoint {
    pub bucket: String,
    pub requested: i64,
    pub verified: i64,
    pub success_rate: f64,
}

// ========== Publishing Trends ==========

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublishingTrendsFilters {
    pub group_by: AnalyticsInterval,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublishingTrendsRequest {
    pub envelope: AnalyticsEnvelope,
    pub filters: PublishingTrendsFilters,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublishingTrendPoint {
    pub bucket: String,
    pub counts: HashMap<String, i64>,
}

// ========== Page Views ==========

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PageViewsFilters {
    pub group_by: AnalyticsInterval,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_id: Option<i32>,
    pub only_unique: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PageViewsRequest {
    pub envelope: AnalyticsEnvelope,
    pub filters: PageViewsFilters,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PageViewPoint {
    pub bucket: String,
    pub views: i64,
    pub unique_visitors: i64,
}

// ========== Comment Rate ==========

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommentRateFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_views: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommentRateRequest {
    pub envelope: AnalyticsEnvelope,
    pub filters: CommentRateFilters,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommentRatePoint {
    pub post_id: i32,
    pub title: String,
    pub views: i64,
    pub comments: i64,
    pub comment_rate: f64,
}

// ========== Newsletter Growth ==========

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NewsletterGrowthFilters {
    pub group_by: AnalyticsInterval,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NewsletterGrowthRequest {
    pub envelope: AnalyticsEnvelope,
    pub filters: NewsletterGrowthFilters,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NewsletterGrowthPoint {
    pub bucket: String,
    pub new_subscribers: i64,
    pub confirmed: i64,
    pub unsubscribed: i64,
    pub net_growth: i64,
}

// ========== Media Upload Trends ==========

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaUploadFilters {
    pub group_by: AnalyticsInterval,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaUploadRequest {
    pub envelope: AnalyticsEnvelope,
    pub filters: MediaUploadFilters,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MediaUploadPoint {
    pub bucket: String,
    pub upload_count: i64,
    pub total_size_mb: f64,
    pub avg_size_mb: f64,
}

// ========== Dashboard Summary ==========

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DashboardSummaryFilters {
    pub period: String, // "7d" | "30d" | "90d"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DashboardSummaryRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub envelope: Option<AnalyticsEnvelope>,
    pub filters: DashboardSummaryFilters,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DashboardSummaryUsers {
    pub total: i64,
    pub new_in_period: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DashboardSummaryPosts {
    pub published: i64,
    pub drafts: i64,
    pub views_in_period: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DashboardSummaryEngagement {
    pub comments_in_period: i64,
    pub newsletter_confirmed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DashboardSummaryMedia {
    pub total_files: i64,
    pub uploads_in_period: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DashboardSummaryData {
    pub users: DashboardSummaryUsers,
    pub posts: DashboardSummaryPosts,
    pub engagement: DashboardSummaryEngagement,
    pub media: DashboardSummaryMedia,
}

// ========== Analytics State ==========

pub struct AnalyticsState {
    pub registration_trends: GlobalSignal<
        StateFrame<
            AnalyticsEnvelopeResponse<Vec<RegistrationTrendPoint>>,
            RegistrationTrendsRequest,
        >,
    >,
    pub verification_rates: GlobalSignal<
        StateFrame<AnalyticsEnvelopeResponse<Vec<VerificationRatePoint>>, VerificationRatesRequest>,
    >,
    pub publishing_trends: GlobalSignal<
        StateFrame<AnalyticsEnvelopeResponse<Vec<PublishingTrendPoint>>, PublishingTrendsRequest>,
    >,
    pub page_views:
        GlobalSignal<StateFrame<AnalyticsEnvelopeResponse<Vec<PageViewPoint>>, PageViewsRequest>>,
    pub comment_rate: GlobalSignal<
        StateFrame<AnalyticsEnvelopeResponse<Vec<CommentRatePoint>>, CommentRateRequest>,
    >,
    pub newsletter_growth: GlobalSignal<
        StateFrame<AnalyticsEnvelopeResponse<Vec<NewsletterGrowthPoint>>, NewsletterGrowthRequest>,
    >,
    pub media_upload: GlobalSignal<
        StateFrame<AnalyticsEnvelopeResponse<Vec<MediaUploadPoint>>, MediaUploadRequest>,
    >,
    pub dashboard_summary: GlobalSignal<
        StateFrame<AnalyticsEnvelopeResponse<DashboardSummaryData>, DashboardSummaryRequest>,
    >,
}

impl AnalyticsState {
    pub fn new() -> Self {
        Self {
            registration_trends: GlobalSignal::new(|| StateFrame::new()),
            verification_rates: GlobalSignal::new(|| StateFrame::new()),
            publishing_trends: GlobalSignal::new(|| StateFrame::new()),
            page_views: GlobalSignal::new(|| StateFrame::new()),
            comment_rate: GlobalSignal::new(|| StateFrame::new()),
            newsletter_growth: GlobalSignal::new(|| StateFrame::new()),
            media_upload: GlobalSignal::new(|| StateFrame::new()),
            dashboard_summary: GlobalSignal::new(|| StateFrame::new()),
        }
    }
}

static ANALYTICS_STATE: OnceLock<AnalyticsState> = OnceLock::new();

pub fn use_analytics() -> &'static AnalyticsState {
    ANALYTICS_STATE.get_or_init(|| AnalyticsState::new())
}
