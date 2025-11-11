# Dashboard/Analytics Charts Integration Plan

This plan connects the existing analytics store (src/store/analytics/) to the Dashboard (Home) and full Analytics screens with chart visualizations using dioxus-charts. Paths include starting line numbers for quick navigation.

Progress legend:
- [x] Completed
- [~] In progress
- [ ] Not started

## Scope
- Use existing analytics store types and actions [x]
  - src/store/analytics/mod.rs:1
  - src/store/analytics/state.rs:1 (shared types), 53 (registration), 72 (verification), 98 (publishing), 124 (page views), 153 (comment rate), 176 (newsletter growth), 200 (media upload), 224 (dashboard summary), 247 (AnalyticsState), 259 (use_analytics)
  - src/store/analytics/actions.rs:1 (all fetch_* actions)
- Render charts on:
  - Dashboard/Home: src/screens/home/mod.rs:1 [~] (components scaffolded, wiring in progress)
  - Analytics page: src/screens/analytics.rs:1 [ ]

## Dependencies
- Add dioxus-charts to Cargo [x]
  - Cargo.toml:1 — added `dioxus-charts = "0.1.0"` (confirm version alignment with crate API)

## Data → Chart Mapping
- Registration Trends (time-series line)
  - Input: Vec<RegistrationTrendPoint> (src/store/analytics/state.rs:66)
  - x: bucket (String), y: new_users (i64)
- Verification Rates (grouped bars + optional line)
  - Input: Vec<VerificationRatePoint> (src/store/analytics/state.rs:90)
  - Bars: requested, verified; Optional line: success_rate
- Publishing Trends (stacked bars per bucket)
  - Input: Vec<PublishingTrendPoint> (src/store/analytics/state.rs:116)
  - Stack keys: counts map values by status labels
- Page Views (multi-line)
  - Input: Vec<PageViewPoint> (src/store/analytics/state.rs:144)
  - Lines: views, unique_visitors
- Comment Rate (horizontal ranking bars)
  - Input: Vec<CommentRatePoint> (src/store/analytics/state.rs:167)
  - Value: comment_rate (or comments) per post title; show top N
- Newsletter Growth (stacked bars + line)
  - Input: Vec<NewsletterGrowthPoint> (src/store/analytics/state.rs:188)
  - Bars: new_subscribers, confirmed, unsubscribed; Line: net_growth
- Media Upload Trends (bars + line)
  - Input: Vec<MediaUploadPoint> (src/store/analytics/state.rs:210)
  - Bars: upload_count; Line: avg_size_mb (with secondary axis if supported)
- Dashboard Summary (KPI cards)
  - Input: DashboardSummaryData (src/store/analytics/state.rs:236)
  - Cards: users.total/new_in_period, posts.published/drafts/views_in_period, engagement.comments_in_period/newsletter_confirmed, media.total_files/uploads_in_period

## Components to Add (reusable)
- Directory: src/components/analytics/
  - registration_trend_chart.rs:1 — Time-series line of new users
  - verification_rates_chart.rs:1 — Grouped bar (requested/verified) + optional success rate line
  - publishing_trends_chart.rs:1 — Stacked bars per bucket by status label
  - page_views_chart.rs:1 — Two-line chart (views, unique_visitors)
  - comment_rate_chart.rs:1 — Horizontal bar ranking by comment_rate or comments
  - newsletter_growth_chart.rs:1 — Stacked bars + net growth line
  - media_upload_trends_chart.rs:1 — Bars for uploads + line for avg size
  - dashboard_summary_cards.rs:1 — KPI cards grid for summary

All chart components:
- Accept minimal typed props (Vec<Point> or DashboardSummaryData) and optional title/height. [x]
- Currently render using lightweight SVG/div scaffolds within Tailwind-styled cards; to be upgraded to dioxus-charts primitives (axes, grid, legend, tooltip) once API is finalized. [~]

## Screen Wiring
- Home (Dashboard): src/screens/home/mod.rs:1
  - Replace placeholder cards with DashboardSummaryCards [~]
  - Panels: PageViewsChart and PublishingTrendsChart wired to `use_analytics` [~]
  - Secondary row: RegistrationTrendChart and VerificationRatesChart (or CommentRateChart) [ ]
- Analytics page: src/screens/analytics.rs:1
  - Full grid with all charts, each with compact filter controls (interval, sort, etc.) [ ]

## Fetch Semantics
- On mount for Dashboard/Home:
  - Call `use_analytics().fetch_dashboard_summary(...)` (src/store/analytics/actions.rs:1)
  - Trigger Page Views and Publishing Trends with default filters (day/week) for quick insights
- On Analytics page mount:
  - Fetch all or lazily per tab/accordion
- Use `StateFrame` status for each signal to drive loading/error content
  - StateFrame helpers: src/store/lib.rs:252, src/store/lib.rs:303

## Filters & Controls
- Common interval selector (hour/day/week/month) per time-series
  - Request types: see filters per endpoint (src/store/analytics/state.rs:55, 74, 101, 127, 179, 201)
- Comment rate selector: sort by comment_rate or comments; min_views input (if backend supports)
- Page views: post_id and author_id fields and `only_unique` toggle
- Dashboard summary: period selector (7d/30d/90d)

## Loading, Errors, Empty States
- Loading skeletons or spinners within chart cards while frame.status == Loading
- Error banners using existing toast and AppError formatting
  - use_state_frame_toast: src/hooks/use_state_frame_toast.rs:1
- Empty state messages if data arrays are empty

## Styling & Theming
- Card shell: rounded-xl, subtle border, translucent bg to match existing placeholders
- Light/dark-aware series colors (e.g., sky, emerald, amber, rose) with Tailwind classes
- Responsive grid: 1-col on small, 2-col on lg, 3–4-col on xl+ as in current layout

## File Changes (planned → progress)
- Cargo.toml:1 — add dioxus-charts dependency [x]
- src/components/analytics/ [x scaffolded]
  - registration_trend_chart.rs:1 [x scaffolded card + basic SVG line]
  - verification_rates_chart.rs:1 [x scaffolded grouped bars + interval controls]
  - publishing_trends_chart.rs:1 [x scaffolded stacked pseudo-bars]
  - page_views_chart.rs:1 [x scaffolded two-line SVG chart]
  - comment_rate_chart.rs:1 [x scaffolded horizontal ranking bars + store-bound variant]
  - newsletter_growth_chart.rs:1 [x scaffolded stacked bars + line placeholder]
  - media_upload_trends_chart.rs:1 [x scaffolded bars + line placeholder]
  - dashboard_summary_cards.rs:1 [x scaffolded KPI cards grid]
- src/screens/home/mod.rs:1 — render summary + 2–3 charts [~ wiring to new components in progress]
- src/screens/analytics.rs:1 — render full chart grid with filters [ ]

## Notes
- The analytics store already exists and matches backend contracts: src/store/analytics/
- Charts should bind to `GlobalSignal<StateFrame<AnalyticsEnvelopeResponse<...>, RequestType>>` members of AnalyticsState (src/store/analytics/state.rs:247) via `use_analytics()` (src/store/analytics/state.rs:259).
