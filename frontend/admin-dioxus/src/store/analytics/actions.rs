use crate::services::http_client;
use crate::store::analytics::*;
use crate::store::{state_request_abstraction, StateFrame};

impl AnalyticsState {
    pub async fn fetch_registration_trends(&self, request: RegistrationTrendsRequest) {
        let req = http_client::post("/analytics/v1/user/registration-trends", &request);

        state_request_abstraction(
            &self.registration_trends,
            Some(request),
            req.send(),
            "registration_trends",
            |response: &AnalyticsEnvelopeResponse<Vec<RegistrationTrendPoint>>| {
                (Some(response.clone()), None)
            },
        )
        .await;
    }

    pub async fn fetch_verification_rates(&self, request: VerificationRatesRequest) {
        let req = http_client::post("/analytics/v1/user/verification-rates", &request);

        state_request_abstraction(
            &self.verification_rates,
            Some(request),
            req.send(),
            "verification_rates",
            |response: &AnalyticsEnvelopeResponse<Vec<VerificationRatePoint>>| {
                (Some(response.clone()), None)
            },
        )
        .await;
    }

    pub async fn fetch_publishing_trends(&self, request: PublishingTrendsRequest) {
        let req = http_client::post("/analytics/v1/content/publishing-trends", &request);

        state_request_abstraction(
            &self.publishing_trends,
            Some(request),
            req.send(),
            "publishing_trends",
            |response: &AnalyticsEnvelopeResponse<Vec<PublishingTrendPoint>>| {
                (Some(response.clone()), None)
            },
        )
        .await;
    }

    pub async fn fetch_page_views(&self, request: PageViewsRequest) {
        let req = http_client::post("/analytics/v1/engagement/page-views", &request);

        state_request_abstraction(
            &self.page_views,
            Some(request),
            req.send(),
            "page_views",
            |response: &AnalyticsEnvelopeResponse<Vec<PageViewPoint>>| {
                (Some(response.clone()), None)
            },
        )
        .await;
    }

    pub async fn fetch_comment_rate(&self, request: CommentRateRequest) {
        let req = http_client::post("/analytics/v1/engagement/comment-rate", &request);

        state_request_abstraction(
            &self.comment_rate,
            Some(request),
            req.send(),
            "comment_rate",
            |response: &AnalyticsEnvelopeResponse<Vec<CommentRatePoint>>| {
                (Some(response.clone()), None)
            },
        )
        .await;
    }

    pub async fn fetch_newsletter_growth(&self, request: NewsletterGrowthRequest) {
        let req = http_client::post("/analytics/v1/engagement/newsletter-growth", &request);

        state_request_abstraction(
            &self.newsletter_growth,
            Some(request),
            req.send(),
            "newsletter_growth",
            |response: &AnalyticsEnvelopeResponse<Vec<NewsletterGrowthPoint>>| {
                (Some(response.clone()), None)
            },
        )
        .await;
    }

    pub async fn fetch_media_upload(&self, request: MediaUploadRequest) {
        let req = http_client::post("/analytics/v1/media/upload-trends", &request);

        state_request_abstraction(
            &self.media_upload,
            Some(request),
            req.send(),
            "media_upload",
            |response: &AnalyticsEnvelopeResponse<Vec<MediaUploadPoint>>| {
                (Some(response.clone()), None)
            },
        )
        .await;
    }

    pub async fn fetch_dashboard_summary(&self, request: DashboardSummaryRequest) {
        let req = http_client::post("/analytics/v1/dashboard/summary", &request);

        state_request_abstraction(
            &self.dashboard_summary,
            Some(request),
            req.send(),
            "dashboard_summary",
            |response: &AnalyticsEnvelopeResponse<DashboardSummaryData>| {
                (Some(response.clone()), None)
            },
        )
        .await;
    }

    pub fn reset(&self) {
        *self.registration_trends.write() = StateFrame::new();
        *self.verification_rates.write() = StateFrame::new();
        *self.publishing_trends.write() = StateFrame::new();
        *self.page_views.write() = StateFrame::new();
        *self.comment_rate.write() = StateFrame::new();
        *self.newsletter_growth.write() = StateFrame::new();
        *self.media_upload.write() = StateFrame::new();
        *self.dashboard_summary.write() = StateFrame::new();
    }
}
