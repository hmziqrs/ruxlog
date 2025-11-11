use dioxus::prelude::*;

use crate::components::analytics::{
    dashboard_summary_cards::DashboardSummaryCards, filter_toolbar::AnalyticsFilterToolbar,
    page_views_chart::PageViewsChart, publishing_trends_chart::PublishingTrendsChart,
    registration_trend_chart::RegistrationTrendChart,
    verification_rates_chart::VerificationRatesChart,
};
use crate::components::PageHeader;
use crate::store::analytics::{
    use_analytics, use_analytics_filters, AnalyticsInterval, DashboardSummaryFilters,
    DashboardSummaryRequest, PageViewsFilters, PageViewsRequest, PublishingTrendsFilters,
    PublishingTrendsRequest, RegistrationTrendsFilters, RegistrationTrendsRequest,
    VerificationRatesFilters, VerificationRatesRequest,
};

#[component]
pub fn HomeScreen() -> Element {
    let analytics = use_analytics();
    let filters = use_analytics_filters();

    // Local state for page views chart-specific filters
    let mut page_views_interval = use_signal(|| AnalyticsInterval::Day);
    let mut page_views_post_id = use_signal(|| None::<i32>);
    let mut page_views_author_id = use_signal(|| None::<i32>);
    let mut page_views_only_unique = use_signal(|| false);

    // Refetch all analytics data using current filter state
    let refetch_all = move || {
        spawn(async move {
            let envelope = filters.build_envelope();

            // Dashboard summary
            let summary_req = DashboardSummaryRequest {
                envelope: Some(envelope.clone()),
                filters: DashboardSummaryFilters {
                    period: filters
                        .period_preset
                        .read()
                        .clone()
                        .unwrap_or_else(|| "7d".to_string()),
                },
            };
            analytics.fetch_dashboard_summary(summary_req).await;

            // Page views
            let page_views_req = PageViewsRequest {
                envelope: envelope.clone(),
                filters: PageViewsFilters {
                    group_by: AnalyticsInterval::Day,
                    post_id: None,
                    author_id: None,
                    only_unique: false,
                },
            };
            analytics.fetch_page_views(page_views_req).await;

            // Publishing trends
            let publishing_req = PublishingTrendsRequest {
                envelope: envelope.clone(),
                filters: PublishingTrendsFilters {
                    group_by: AnalyticsInterval::Day,
                    status: None,
                },
            };
            analytics.fetch_publishing_trends(publishing_req).await;

            // Registration trends
            let registration_req = RegistrationTrendsRequest {
                envelope: envelope.clone(),
                filters: RegistrationTrendsFilters {
                    group_by: AnalyticsInterval::Day,
                },
            };
            analytics.fetch_registration_trends(registration_req).await;

            // Verification rates
            let verification_req = VerificationRatesRequest {
                envelope: envelope.clone(),
                filters: VerificationRatesFilters {
                    group_by: AnalyticsInterval::Day,
                },
            };
            analytics.fetch_verification_rates(verification_req).await;
        });
    };

    // Kick off initial dashboard analytics fetches on mount.
    use_future(move || async move {
        refetch_all();
    });

    // Read frames once for rendering; the inner components handle states.
    let summary_frame = analytics.dashboard_summary.read();
    let page_views_frame = analytics.page_views.read();
    let publishing_frame = analytics.publishing_trends.read();
    let registration_frame = analytics.registration_trends.read();
    let verification_frame = analytics.verification_rates.read();

    rsx! {
        div { class: "min-h-screen bg-transparent text-foreground",
            // Page header
            PageHeader {
                title: "Analytics overview".to_string(),
                description: "Key metrics for users, content, engagement, and media.".to_string(),
            }

            // Filter toolbar
            AnalyticsFilterToolbar {
                on_filter_change: move |_| {
                    refetch_all();
                },
            }

            div { class: "container mx-auto px-4 my-6 space-y-6",

                // Summary KPI cards row
                DashboardSummaryCards {
                    frame: summary_frame.clone(),
                }

                // Primary charts row: traffic and publishing
                div { class: "grid grid-cols-1 lg:grid-cols-2 gap-4",
                    PageViewsChart {
                        frame: page_views_frame.clone(),
                        title: "Traffic & views".to_string(),
                        height: "h-72".to_string(),
                        compact: false,
                        current_interval: *page_views_interval.read(),
                        on_interval_change: Some(EventHandler::new(move |interval: AnalyticsInterval| {
                            *page_views_interval.write() = interval;
                            spawn(async move {
                                let envelope = filters.build_envelope();
                                let req = PageViewsRequest {
                                    envelope,
                                    filters: PageViewsFilters {
                                        group_by: interval,
                                        post_id: *page_views_post_id.read(),
                                        author_id: *page_views_author_id.read(),
                                        only_unique: *page_views_only_unique.read(),
                                    },
                                };
                                analytics.fetch_page_views(req).await;
                            });
                        })),
                        current_post_id: *page_views_post_id.read(),
                        on_post_id_change: Some(EventHandler::new(move |post_id: Option<i32>| {
                            *page_views_post_id.write() = post_id;
                            spawn(async move {
                                let envelope = filters.build_envelope();
                                let req = PageViewsRequest {
                                    envelope,
                                    filters: PageViewsFilters {
                                        group_by: *page_views_interval.read(),
                                        post_id,
                                        author_id: *page_views_author_id.read(),
                                        only_unique: *page_views_only_unique.read(),
                                    },
                                };
                                analytics.fetch_page_views(req).await;
                            });
                        })),
                        current_author_id: *page_views_author_id.read(),
                        on_author_id_change: Some(EventHandler::new(move |author_id: Option<i32>| {
                            *page_views_author_id.write() = author_id;
                            spawn(async move {
                                let envelope = filters.build_envelope();
                                let req = PageViewsRequest {
                                    envelope,
                                    filters: PageViewsFilters {
                                        group_by: *page_views_interval.read(),
                                        post_id: *page_views_post_id.read(),
                                        author_id,
                                        only_unique: *page_views_only_unique.read(),
                                    },
                                };
                                analytics.fetch_page_views(req).await;
                            });
                        })),
                        current_only_unique: *page_views_only_unique.read(),
                        on_only_unique_change: Some(EventHandler::new(move |only_unique: bool| {
                            *page_views_only_unique.write() = only_unique;
                            spawn(async move {
                                let envelope = filters.build_envelope();
                                let req = PageViewsRequest {
                                    envelope,
                                    filters: PageViewsFilters {
                                        group_by: *page_views_interval.read(),
                                        post_id: *page_views_post_id.read(),
                                        author_id: *page_views_author_id.read(),
                                        only_unique,
                                    },
                                };
                                analytics.fetch_page_views(req).await;
                            });
                        })),
                    }

                    PublishingTrendsChart {
                        frame: publishing_frame.clone(),
                        title: "Publishing activity".to_string(),
                        height_class: "h-72".to_string(),
                        description: Some("Posts by status across recent days.".to_string()),
                    }
                }

                // Secondary charts row: registrations & verifications
                div { class: "grid grid-cols-1 lg:grid-cols-2 gap-4",
                    RegistrationTrendChart {
                        frame: registration_frame.clone(),
                        title: "New user registrations (last 7 days)".to_string(),
                        height: "h-64".to_string(),
                    }

                    VerificationRatesChart {
                        frame: verification_frame.clone(),
                        title: "Verification funnel".to_string(),
                        height: "260px".to_string(),
                        show_success_rate: true,
                        on_interval_change: Some(EventHandler::new(move |interval: AnalyticsInterval| {
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
                        })),
                    }
                }
            }
        }
    }
}
