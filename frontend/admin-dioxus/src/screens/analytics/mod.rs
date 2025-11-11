use dioxus::prelude::*;

use crate::components::analytics::{
    comment_rate_chart::CommentRateChartFromStore, dashboard_summary_cards::DashboardSummaryCards,
    filter_toolbar::AnalyticsFilterToolbar, media_upload_trends_chart::MediaUploadTrendsChart,
    newsletter_growth_chart::NewsletterGrowthChart, page_views_chart::PageViewsChart,
    publishing_trends_chart::PublishingTrendsChart,
    registration_trend_chart::RegistrationTrendChart,
    verification_rates_chart::VerificationRatesChart,
};
use crate::components::PageHeader;
use crate::store::analytics::{
    use_analytics, use_analytics_filters, AnalyticsInterval, CommentRateFilters,
    CommentRateRequest, DashboardSummaryFilters, DashboardSummaryRequest, MediaUploadFilters,
    MediaUploadRequest, NewsletterGrowthFilters, NewsletterGrowthRequest, PageViewsFilters,
    PageViewsRequest, PublishingTrendsFilters, PublishingTrendsRequest, RegistrationTrendsFilters,
    RegistrationTrendsRequest, VerificationRatesFilters, VerificationRatesRequest,
};

/// Full analytics screen:
/// - Shows summary KPIs
/// - Full grid of all analytics charts
/// - Compact filters per chart (interval, sort, etc.)
/// - Uses analytics store `use_analytics` + `use_analytics_filters`
/// - Mirrors/extends Home dashboard wiring as per `analytics-dashboard-charts-plan.md`.
#[component]
pub fn AnalyticsScreen() -> Element {
    let analytics = use_analytics();
    let filters = use_analytics_filters();

    //
    // Local UI state for per-chart filters.
    // These are "compact controls" layered on top of the shared analytics filters.
    //

    // Summary period selector (7d/30d/90d)
    let summary_period = use_signal(|| "7d".to_string());

    // Page views chart filters
    let mut pv_interval = use_signal(|| AnalyticsInterval::Day);
    let mut pv_post_id = use_signal(|| None::<i32>);
    let mut pv_author_id = use_signal(|| None::<i32>);
    let mut pv_only_unique = use_signal(|| false);

    // Publishing trends filters
    let publishing_interval = use_signal(|| AnalyticsInterval::Day);

    // Registration trends interval
    let registration_interval = use_signal(|| AnalyticsInterval::Day);

    // Verification rates interval
    let mut verification_interval = use_signal(|| AnalyticsInterval::Day);

    // Comment rate filters (placeholder for future enhancements using min_views/sort, currently unused)

    // Newsletter growth interval
    let newsletter_interval = use_signal(|| AnalyticsInterval::Day);

    // Media upload trends interval
    let media_interval = use_signal(|| AnalyticsInterval::Day);

    //
    // Initial fetch on mount:
    // Fetch all analytics series so the screen loads with data.
    //
    use_future({
        let analytics = analytics;
        let filters = filters;
        move || async move {
            let envelope = filters.build_envelope();

            // Summary
            let summary_req = DashboardSummaryRequest {
                envelope: Some(envelope.clone()),
                filters: DashboardSummaryFilters {
                    period: summary_period.read().clone(),
                },
            };
            analytics.fetch_dashboard_summary(summary_req).await;

            // Page views
            let page_views_req = PageViewsRequest {
                envelope: envelope.clone(),
                filters: PageViewsFilters {
                    group_by: *pv_interval.read(),
                    post_id: *pv_post_id.read(),
                    author_id: *pv_author_id.read(),
                    only_unique: *pv_only_unique.read(),
                },
            };
            analytics.fetch_page_views(page_views_req).await;

            // Publishing trends
            let publishing_req = PublishingTrendsRequest {
                envelope: envelope.clone(),
                filters: PublishingTrendsFilters {
                    group_by: *publishing_interval.read(),
                    status: None,
                },
            };
            analytics.fetch_publishing_trends(publishing_req).await;

            // Registration trends
            let registration_req = RegistrationTrendsRequest {
                envelope: envelope.clone(),
                filters: RegistrationTrendsFilters {
                    group_by: *registration_interval.read(),
                },
            };
            analytics.fetch_registration_trends(registration_req).await;

            // Verification rates
            let verification_req = VerificationRatesRequest {
                envelope: envelope.clone(),
                filters: VerificationRatesFilters {
                    group_by: *verification_interval.read(),
                },
            };
            analytics.fetch_verification_rates(verification_req).await;

            // Comment rate
            let comment_req = CommentRateRequest {
                envelope: envelope.clone(),
                filters: CommentRateFilters { min_views: None },
            };
            analytics.fetch_comment_rate(comment_req).await;

            // Newsletter growth
            let newsletter_req = NewsletterGrowthRequest {
                envelope: envelope.clone(),
                filters: NewsletterGrowthFilters {
                    group_by: *newsletter_interval.read(),
                },
            };
            analytics.fetch_newsletter_growth(newsletter_req).await;

            // Media uploads
            let media_req = MediaUploadRequest {
                envelope,
                filters: MediaUploadFilters {
                    group_by: *media_interval.read(),
                },
            };
            analytics.fetch_media_upload(media_req).await;
        }
    });

    //
    // Snapshot frames for rendering.
    // Each chart component is responsible for loading/empty visuals based on the frame.
    //
    let summary_frame = analytics.dashboard_summary.read();
    let page_views_frame = analytics.page_views.read();
    let publishing_frame = analytics.publishing_trends.read();
    let registration_frame = analytics.registration_trends.read();
    let verification_frame = analytics.verification_rates.read();
    let media_upload_frame = analytics.media_upload.read();

    rsx! {
        div { class: "min-h-screen bg-transparent text-foreground",
            PageHeader {
                title: "Analytics".to_string(),
                description: "Deep-dive into traffic, publishing, engagement, and growth signals."
                    .to_string(),
            }

            // Global analytics filter toolbar (date range, presets, etc.).
            // On change: rebuild envelope and refetch relevant frames.
            AnalyticsFilterToolbar {
                on_filter_change: move |_| {
                    let analytics = analytics;
                    let filters = filters;
                    spawn(async move {
                        let envelope = filters.build_envelope();

                        // Summary (preserve period)
                        let summary_req = DashboardSummaryRequest {
                            envelope: Some(envelope.clone()),
                            filters: DashboardSummaryFilters {
                                period: summary_period.read().clone(),
                            },
                        };
                        analytics.fetch_dashboard_summary(summary_req).await;

                        // Page views
                        let page_views_req = PageViewsRequest {
                            envelope: envelope.clone(),
                            filters: PageViewsFilters {
                                group_by: *pv_interval.read(),
                                post_id: *pv_post_id.read(),
                                author_id: *pv_author_id.read(),
                                only_unique: *pv_only_unique.read(),
                            },
                        };
                        analytics.fetch_page_views(page_views_req).await;

                        // Publishing
                        let publishing_req = PublishingTrendsRequest {
                            envelope: envelope.clone(),
                            filters: PublishingTrendsFilters {
                                group_by: *publishing_interval.read(),
                                status: None,
                            },
                        };
                        analytics.fetch_publishing_trends(publishing_req).await;

                        // Registration
                        let registration_req = RegistrationTrendsRequest {
                            envelope: envelope.clone(),
                            filters: RegistrationTrendsFilters {
                                group_by: *registration_interval.read(),
                            },
                        };
                        analytics.fetch_registration_trends(registration_req).await;

                        // Verification
                        let verification_req = VerificationRatesRequest {
                            envelope: envelope.clone(),
                            filters: VerificationRatesFilters {
                                group_by: *verification_interval.read(),
                            },
                        };
                        analytics.fetch_verification_rates(verification_req).await;

                        // Comment rate
                        let comment_req = CommentRateRequest {
                            envelope: envelope.clone(),
                            filters: CommentRateFilters {
                                min_views: None,
                            },
                        };
                        analytics.fetch_comment_rate(comment_req).await;

                        // Newsletter growth
                        let newsletter_req = NewsletterGrowthRequest {
                            envelope: envelope.clone(),
                            filters: NewsletterGrowthFilters {
                                group_by: *newsletter_interval.read(),
                            },
                        };
                        analytics.fetch_newsletter_growth(newsletter_req).await;

                        // Media uploads
                        let media_req = MediaUploadRequest {
                            envelope,
                            filters: MediaUploadFilters {
                                group_by: *media_interval.read(),
                            },
                        };
                        analytics.fetch_media_upload(media_req).await;
                    });
                },
            }

            div { class: "container mx-auto px-4 my-6 space-y-6",

                // Summary KPIs
                DashboardSummaryCards {
                    frame: summary_frame.clone(),
                }

                // Row 1: Traffic & publishing
                div { class: "grid grid-cols-1 xl:grid-cols-2 gap-4",
                    PageViewsChart {
                        frame: page_views_frame.clone(),
                        title: "Page views & visitors".to_string(),
                        height: "h-80".to_string(),
                        compact: false,
                        current_interval: *pv_interval.read(),
                        on_interval_change: Some(EventHandler::new({
                            let analytics = analytics;
                            let filters = filters;
                            move |interval: AnalyticsInterval| {
                                *pv_interval.write() = interval;
                                let analytics = analytics;
                                let filters = filters;
                                spawn(async move {
                                    let envelope = filters.build_envelope();
                                    let req = PageViewsRequest {
                                        envelope,
                                        filters: PageViewsFilters {
                                            group_by: interval,
                                            post_id: *pv_post_id.read(),
                                            author_id: *pv_author_id.read(),
                                            only_unique: *pv_only_unique.read(),
                                        },
                                    };
                                    analytics.fetch_page_views(req).await;
                                });
                            }
                        })),
                        current_post_id: *pv_post_id.read(),
                        on_post_id_change: Some(EventHandler::new({
                            let analytics = analytics;
                            let filters = filters;
                            move |post_id: Option<i32>| {
                                *pv_post_id.write() = post_id;
                                let analytics = analytics;
                                let filters = filters;
                                spawn(async move {
                                    let envelope = filters.build_envelope();
                                    let req = PageViewsRequest {
                                        envelope,
                                        filters: PageViewsFilters {
                                            group_by: *pv_interval.read(),
                                            post_id,
                                            author_id: *pv_author_id.read(),
                                            only_unique: *pv_only_unique.read(),
                                        },
                                    };
                                    analytics.fetch_page_views(req).await;
                                });
                            }
                        })),
                        current_author_id: *pv_author_id.read(),
                        on_author_id_change: Some(EventHandler::new({
                            let analytics = analytics;
                            let filters = filters;
                            move |author_id: Option<i32>| {
                                *pv_author_id.write() = author_id;
                                let analytics = analytics;
                                let filters = filters;
                                spawn(async move {
                                    let envelope = filters.build_envelope();
                                    let req = PageViewsRequest {
                                        envelope,
                                        filters: PageViewsFilters {
                                            group_by: *pv_interval.read(),
                                            post_id: *pv_post_id.read(),
                                            author_id,
                                            only_unique: *pv_only_unique.read(),
                                        },
                                    };
                                    analytics.fetch_page_views(req).await;
                                });
                            }
                        })),
                        current_only_unique: *pv_only_unique.read(),
                        on_only_unique_change: Some(EventHandler::new({
                            let analytics = analytics;
                            let filters = filters;
                            move |only_unique: bool| {
                                *pv_only_unique.write() = only_unique;
                                let analytics = analytics;
                                let filters = filters;
                                spawn(async move {
                                    let envelope = filters.build_envelope();
                                    let req = PageViewsRequest {
                                        envelope,
                                        filters: PageViewsFilters {
                                            group_by: *pv_interval.read(),
                                            post_id: *pv_post_id.read(),
                                            author_id: *pv_author_id.read(),
                                            only_unique,
                                        },
                                    };
                                    analytics.fetch_page_views(req).await;
                                });
                            }
                        })),
                    }

                    PublishingTrendsChart {
                        frame: publishing_frame.clone(),
                        title: "Publishing trends by status".to_string(),
                        height_class: "h-80".to_string(),
                        description: Some("Stacked view of posts across published, drafts, scheduled, and other statuses.".to_string()),
                    }
                }

                // Row 2: User funnel & verification
                div { class: "grid grid-cols-1 xl:grid-cols-2 gap-4",
                    RegistrationTrendChart {
                        frame: registration_frame.clone(),
                        title: "User registrations over time".to_string(),
                        height: "h-72".to_string(),
                    }

                    VerificationRatesChart {
                        frame: verification_frame.clone(),
                        title: "Verification funnel & success rate".to_string(),
                        height: "h-72".to_string(),
                        show_success_rate: true,
                        on_interval_change: Some(EventHandler::new({
                            let analytics = analytics;
                            let filters = filters;
                            move |interval: AnalyticsInterval| {
                                *verification_interval.write() = interval;
                                let analytics = analytics;
                                let filters = filters;
                                spawn(async move {
                                    let envelope = filters.build_envelope();
                                    let req = VerificationRatesRequest {
                                        envelope,
                                        filters: VerificationRatesFilters {
                                            group_by: interval,
                                        },
                                    };
                                    analytics.fetch_verification_rates(req).await;
                                });
                            }
                        })),
                    }
                }

                // Row 3: Engagement ranking & newsletter growth
                div { class: "grid grid-cols-1 xl:grid-cols-2 gap-4",
                    CommentRateChartFromStore {
                        title: Some("Top posts by comment activity".to_string()),
                        height: Some("h-72".to_string()),
                        max_items: Some(10),
                    }

                    NewsletterGrowthChart {
                        title: "Newsletter growth & churn".to_string(),
                        height_class: "h-72".to_string(),
                        default_interval: AnalyticsInterval::Day,
                    }
                }

                // Row 4: Media uploads
                div { class: "grid grid-cols-1 xl:grid-cols-2 gap-4",
                    MediaUploadTrendsChart {
                        frame: media_upload_frame.clone(),
                        title: "Media upload trends".to_string(),
                        height_class: "h-72".to_string(),
                    }
                }
            }
        }
    }
}
