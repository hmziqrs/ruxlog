# Analytics Store Plan (Frontend)

**Status: ✅ COMPLETED**

This document defines the plan and API contracts for the analytics store, mirroring the backend module and following existing store/service patterns. References use file path with starting line numbers.

## Plan
- ✅ Add `src/store/analytics/` with `mod.rs`, `state.rs`, `actions.rs`.
- ✅ Mirror shared request/response types: `AnalyticsEnvelope`, `AnalyticsInterval`, `AnalyticsMeta`, `AnalyticsEnvelopeResponse<T>`.
- ✅ Define per-endpoint request filter types and response point types.
- ✅ Add `AnalyticsState` with one `GlobalSignal<StateFrame<...>>` per endpoint.
- ✅ Implement actions using `http_client::post` and `state_request_abstraction` to parse `AnalyticsEnvelopeResponse<T>`.
- ✅ Re-export analytics store in `src/store/mod.rs` and wire usage into `src/screens/analytics.rs` as needed.

Supporting patterns:
- Request helpers: `state_request_abstraction` and `list_state_abstraction` (src/store/lib.rs:252, src/store/lib.rs:303)
- HTTP client: `http_client::post` (src/services/http_client.rs:15)
- State frame shape and error handling (src/store/lib.rs:13)

## Backend Endpoints
Base router `POST /analytics/v1/...` and protected layers (docs/raw-analytics.md:13):
- `/user/registration-trends` (docs/raw-analytics.md:17)
- `/user/verification-rates` (docs/raw-analytics.md:21)
- `/content/publishing-trends` (docs/raw-analytics.md:25)
- `/engagement/page-views` (docs/raw-analytics.md:28)
- `/engagement/comment-rate` (docs/raw-analytics.md:29)
- `/engagement/newsletter-growth` (docs/raw-analytics.md:31)
- `/media/upload-trends` (docs/raw-analytics.md:35)
- `/dashboard/summary` (docs/raw-analytics.md:38)

## Shared Types
- Request envelope `AnalyticsEnvelope` (docs/raw-analytics.md:1133)
  - date_from: string (YYYY-MM-DD or RFC3339), optional
  - date_to: string, optional
  - page: u64, optional (>= 1)
  - per_page: u64, optional (1..=200)
  - sort_by: string, optional (endpoint-specific)
  - sort_order: "asc" | "desc", optional
- Interval `AnalyticsInterval` = "hour" | "day" | "week" | "month" (docs/raw-analytics.md:1291)
- Response wrapper `AnalyticsEnvelopeResponse<T>` (docs/raw-analytics.md:1765)
  - data: T
  - meta: `AnalyticsMeta`
- Meta `AnalyticsMeta` (docs/raw-analytics.md:1721)
  - total: u64, page: u64, per_page: u64
  - interval: string, optional
  - sorted_by: string, optional
  - filters_applied: object, optional

## Endpoint Contracts (Body & Response)

Registration Trends
- POST `/analytics/v1/user/registration-trends` (docs/raw-analytics.md:17, docs/raw-analytics.md:1928)
- Body: `RegistrationTrendsRequest` (docs/raw-analytics.md:1347)
  - envelope: `AnalyticsEnvelope`
  - filters: { group_by: `AnalyticsInterval` } (docs/raw-analytics.md:1333)
  - sort_by accepted: "new_users" | "bucket" | "count"
- Response: `AnalyticsEnvelopeResponse<Vec<RegistrationTrendPoint>>` (docs/raw-analytics.md:1765)
  - RegistrationTrendPoint: { bucket: string, new_users: i64 } (docs/raw-analytics.md:1362)

Verification Rates
- POST `/analytics/v1/user/verification-rates` (docs/raw-analytics.md:21, docs/raw-analytics.md:1839)
- Body: `VerificationRatesRequest` (docs/raw-analytics.md:1382)
  - envelope: `AnalyticsEnvelope`
  - filters: { group_by: `AnalyticsInterval` } (docs/raw-analytics.md:1368)
  - sort_by accepted: "requested" | "verified" | "success_rate" | "bucket"
- Response: `AnalyticsEnvelopeResponse<Vec<VerificationRatePoint>>` (docs/raw-analytics.md:1765)
  - VerificationRatePoint: { bucket: string, requested: i64, verified: i64, success_rate: f64 } (docs/raw-analytics.md:1397)

Publishing Trends
- POST `/analytics/v1/content/publishing-trends` (docs/raw-analytics.md:25, docs/raw-analytics.md:1890)
- Body: `PublishingTrendsRequest` (docs/raw-analytics.md:1422)
  - envelope: `AnalyticsEnvelope`
  - filters: { group_by: `AnalyticsInterval`, status?: Vec<string> } (docs/raw-analytics.md:1405)
- Response: `AnalyticsEnvelopeResponse<Vec<PublishingTrendPoint>>` (docs/raw-analytics.md:1765)
  - PublishingTrendPoint: { bucket: string, counts: { [status_label]: i64 } } (docs/raw-analytics.md:1437)

Page Views
- POST `/analytics/v1/engagement/page-views` (docs/raw-analytics.md:28, docs/raw-analytics.md:1875)
- Body: `PageViewsRequest` (docs/raw-analytics.md:1468)
  - envelope: `AnalyticsEnvelope`
  - filters: { group_by: `AnalyticsInterval`, post_id?: i32, author_id?: i32, only_unique: bool } (docs/raw-analytics.md:1443)
- Response: `AnalyticsEnvelopeResponse<Vec<PageViewPoint>>` (docs/raw-analytics.md:1765)
  - PageViewPoint: { bucket: string, views: i64, unique_visitors: i64 } (docs/raw-analytics.md:1483)

Comment Rate
- POST `/analytics/v1/engagement/comment-rate` (docs/raw-analytics.md:29, docs/raw-analytics.md:1861)
- Body: `CommentRateRequest` (docs/raw-analytics.md:1526)
  - envelope: `AnalyticsEnvelope`
  - filters: { min_views: i64 (default 100), sort_by: "comment_rate" | "comments" } (docs/raw-analytics.md:1507)
- Response: `AnalyticsEnvelopeResponse<Vec<CommentRatePoint>>` (docs/raw-analytics.md:1765)
  - CommentRatePoint: { post_id: i32, title: string, views: i64, comments: i64, comment_rate: f64 } (docs/raw-analytics.md:1539)

Newsletter Growth
- POST `/analytics/v1/engagement/newsletter-growth` (docs/raw-analytics.md:31)
- Body: `NewsletterGrowthRequest` (docs/raw-analytics.md:1564)
  - envelope: `AnalyticsEnvelope`
  - filters: { group_by: `AnalyticsInterval` } (docs/raw-analytics.md:1548)
- Response: `AnalyticsEnvelopeResponse<Vec<NewsletterGrowthPoint>>` (docs/raw-analytics.md:1765)
  - NewsletterGrowthPoint: { bucket: string, new_subscribers: i64, confirmed: i64, unsubscribed: i64, net_growth: i64 } (docs/raw-analytics.md:1577)

Media Upload Trends
- POST `/analytics/v1/media/upload-trends` (docs/raw-analytics.md:35)
- Body: `MediaUploadRequest` (docs/raw-analytics.md:1602)
  - envelope: `AnalyticsEnvelope`
  - filters: { group_by: `AnalyticsInterval` } (docs/raw-analytics.md:1586)
- Response: `AnalyticsEnvelopeResponse<Vec<MediaUploadPoint>>` (docs/raw-analytics.md:1765)
  - MediaUploadPoint: { bucket: string, upload_count: i64, total_size_mb: f64, avg_size_mb: f64 } (docs/raw-analytics.md:1615)

Dashboard Summary
- POST `/analytics/v1/dashboard/summary` (docs/raw-analytics.md:38, docs/raw-analytics.md:1807)
- Body: `DashboardSummaryRequest` (docs/raw-analytics.md:1671)
  - envelope: Option<`AnalyticsEnvelope`>
  - filters: { period: "7d" | "30d" | "90d" } (docs/raw-analytics.md:1657)
- Response: `AnalyticsEnvelopeResponse<DashboardSummaryData>` (docs/raw-analytics.md:1765)
  - DashboardSummaryData: users, posts, engagement, media (docs/raw-analytics.md:1713)
  - Users: { total: i64, new_in_period: i64 } (docs/raw-analytics.md:1688)
  - Posts: { published: i64, drafts: i64, views_in_period: i64 } (docs/raw-analytics.md:1694)
  - Engagement: { comments_in_period: i64, newsletter_confirmed: i64 } (docs/raw-analytics.md:1701)
  - Media: { total_files: i64, uploads_in_period: i64 } (docs/raw-analytics.md:1707)

## Store Wiring
- ✅ Create `AnalyticsState` with one signal per endpoint, storing the full response to retain meta:
  - Example shape: `GlobalSignal<StateFrame<AnalyticsEnvelopeResponse<Vec<Point>>, RequestType>>`.
- ✅ Use `http_client::post` (src/services/http_client.rs:15) to send requests with body payloads above.
- ✅ Use `state_request_abstraction` (src/store/lib.rs:303) to parse responses into the state frame. Storing the full envelope (`AnalyticsEnvelopeResponse<T>`) avoids losing `meta`.
- ✅ Follow patterns from other stores for structure and OnceLock singletons (src/store/analytics/state.rs:293).

## File Layout
- ✅ `src/store/analytics/mod.rs` — re-exports state/actions
- ✅ `src/store/analytics/state.rs` — types + `AnalyticsState` (src/store/analytics/state.rs:1)
- ✅ `src/store/analytics/actions.rs` — API calls using helpers (src/store/analytics/actions.rs:1)
- ✅ Add `mod analytics;` and `pub use analytics::*;` in `src/store/mod.rs:1`
- Page shell exists at `src/screens/analytics.rs:1` for future UI wiring

### Implementation Details
All 8 endpoints implemented with action methods:
- ✅ fetch_registration_trends (src/store/analytics/actions.rs:6)
- ✅ fetch_verification_rates (src/store/analytics/actions.rs:20)
- ✅ fetch_publishing_trends (src/store/analytics/actions.rs:34)
- ✅ fetch_page_views (src/store/analytics/actions.rs:48)
- ✅ fetch_comment_rate (src/store/analytics/actions.rs:62)
- ✅ fetch_newsletter_growth (src/store/analytics/actions.rs:76)
- ✅ fetch_media_upload (src/store/analytics/actions.rs:90)
- ✅ fetch_dashboard_summary (src/store/analytics/actions.rs:104)

Compiles successfully with no errors.

## Sample Responses (for reference)
- See examples in docs/raw-analytics.md for:
  - Dashboard summary (docs/raw-analytics.md:1807)
  - Verification rates (docs/raw-analytics.md:1839)
  - Comment rate (docs/raw-analytics.md:1861)
  - Page views (docs/raw-analytics.md:1875)
  - Publishing trends (docs/raw-analytics.md:1890)
  - Registration trends (docs/raw-analytics.md:1928)

