use dioxus::prelude::*;

use crate::containers::page_header::PageHeader;

#[cfg(feature = "analytics")]
use crate::containers::analytics::{
    dashboard_summary_cards::DashboardSummaryCards, filter_toolbar::AnalyticsFilterToolbar,
    page_views_chart::PageViewsChart, publishing_trends_chart::PublishingTrendsChart,
};

#[cfg(feature = "analytics")]
use ruxlog_shared::store::analytics::{
    use_analytics, use_analytics_filters, AnalyticsInterval, DashboardSummaryFilters,
    DashboardSummaryRequest, PageViewsFilters, PageViewsRequest, PublishingTrendsFilters,
    PublishingTrendsRequest,
};

#[component]
pub fn HomeScreen() -> Element {
    #[cfg(feature = "analytics")]
    {
        analytics_dashboard()
    }

    #[cfg(not(feature = "analytics"))]
    {
        basic_dashboard()
    }
}

// Mock data structures for dashboard widgets
#[derive(Debug, Clone)]
struct MockComment {
    author: String,
    post_title: String,
    date: String,
}

fn mock_recent_comments() -> Vec<MockComment> {
    vec![
        MockComment {
            author: "Alice Johnson".to_string(),
            post_title: "Getting Started with Rust".to_string(),
            date: "2026-05-12".to_string(),
        },
        MockComment {
            author: "Bob Martinez".to_string(),
            post_title: "Understanding Axum Middleware".to_string(),
            date: "2026-05-11".to_string(),
        },
        MockComment {
            author: "Carol Nguyen".to_string(),
            post_title: "Deploying to Cloudflare Workers".to_string(),
            date: "2026-05-10".to_string(),
        },
        MockComment {
            author: "Dave Smith".to_string(),
            post_title: "Dioxus 0.6 Release Notes".to_string(),
            date: "2026-05-09".to_string(),
        },
        MockComment {
            author: "Eve Williams".to_string(),
            post_title: "Type-Safe APIs with Drizzle".to_string(),
            date: "2026-05-08".to_string(),
        },
    ]
}

// Basic dashboard for minimal blog mode
#[cfg(not(feature = "analytics"))]
fn basic_dashboard() -> Element {
    use crate::router::Route;
    use hmziq_dioxus_free_icons::{icons::ld_icons::*, Icon};

    let nav = use_navigator();
    let recent_comments = mock_recent_comments();

    rsx! {
        div { class: "min-h-screen bg-transparent text-foreground",
            PageHeader {
                title: "Dashboard".to_string(),
                description: "Welcome to Ruxlog admin panel".to_string(),
            }

            div { class: "container mx-auto px-4 my-8",
                // Stats overview row
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-8",
                    // Total Posts
                    div { class: "rounded-lg border border-border/70 bg-card p-5",
                        div { class: "flex items-center justify-between mb-2",
                            p { class: "text-sm font-medium text-muted-foreground", "Total Posts" }
                            div { class: "w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center",
                                Icon { icon: LdFileText, width: 16, height: 16, fill: "currentColor" }
                            }
                        }
                        p { class: "text-2xl font-bold", "24" }
                        p { class: "text-xs text-muted-foreground mt-1", "Published blog posts" }
                    }
                    // Total Comments
                    div { class: "rounded-lg border border-border/70 bg-card p-5",
                        div { class: "flex items-center justify-between mb-2",
                            p { class: "text-sm font-medium text-muted-foreground", "Total Comments" }
                            div { class: "w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center",
                                Icon { icon: LdMessageSquare, width: 16, height: 16, fill: "currentColor" }
                            }
                        }
                        p { class: "text-2xl font-bold", "152" }
                        p { class: "text-xs text-muted-foreground mt-1", "Across all posts" }
                    }
                    // Total Users
                    div { class: "rounded-lg border border-border/70 bg-card p-5",
                        div { class: "flex items-center justify-between mb-2",
                            p { class: "text-sm font-medium text-muted-foreground", "Total Users" }
                            div { class: "w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center",
                                Icon { icon: LdUser, width: 16, height: 16, fill: "currentColor" }
                            }
                        }
                        p { class: "text-2xl font-bold", "38" }
                        p { class: "text-xs text-muted-foreground mt-1", "Registered users" }
                    }
                }

                // Quick action cards
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-8",
                    // Posts card
                    div {
                        class: "p-6 rounded-lg border border-border/70 bg-card hover:bg-accent/50 cursor-pointer transition-colors",
                        onclick: move |_| { nav.push(Route::PostsListScreen {}); },
                        div { class: "flex items-center gap-3 mb-2",
                            div { class: "w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center",
                                Icon { icon: LdFileText, width: 20, height: 20, fill: "currentColor" }
                            }
                            h3 { class: "text-lg font-semibold", "Posts" }
                        }
                        p { class: "text-sm text-muted-foreground", "Create and manage blog posts" }
                    }

                    // Categories card
                    div {
                        class: "p-6 rounded-lg border border-border/70 bg-card hover:bg-accent/50 cursor-pointer transition-colors",
                        onclick: move |_| { nav.push(Route::CategoriesListScreen {}); },
                        div { class: "flex items-center gap-3 mb-2",
                            div { class: "w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center",
                                Icon { icon: LdFolder, width: 20, height: 20, fill: "currentColor" }
                            }
                            h3 { class: "text-lg font-semibold", "Categories" }
                        }
                        p { class: "text-sm text-muted-foreground", "Organize posts by category" }
                    }

                    // Tags card
                    div {
                        class: "p-6 rounded-lg border border-border/70 bg-card hover:bg-accent/50 cursor-pointer transition-colors",
                        onclick: move |_| { nav.push(Route::TagsListScreen {}); },
                        div { class: "flex items-center gap-3 mb-2",
                            div { class: "w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center",
                                Icon { icon: LdTag, width: 20, height: 20, fill: "currentColor" }
                            }
                            h3 { class: "text-lg font-semibold", "Tags" }
                        }
                        p { class: "text-sm text-muted-foreground", "Tag posts for easy discovery" }
                    }

                    // Media card
                    div {
                        class: "p-6 rounded-lg border border-border/70 bg-card hover:bg-accent/50 cursor-pointer transition-colors",
                        onclick: move |_| { nav.push(Route::MediaListScreen {}); },
                        div { class: "flex items-center gap-3 mb-2",
                            div { class: "w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center",
                                Icon { icon: LdImage, width: 20, height: 20, fill: "currentColor" }
                            }
                            h3 { class: "text-lg font-semibold", "Media" }
                        }
                        p { class: "text-sm text-muted-foreground", "Upload and manage images" }
                    }
                }

                // Quick Actions section
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-8",
                    div {
                        class: "flex items-center gap-3 p-4 rounded-lg border border-border/70 bg-card hover:bg-accent/50 cursor-pointer transition-colors",
                        onclick: move |_| { nav.push(Route::PostsAddScreen {}); },
                        div { class: "w-10 h-10 rounded-full bg-green-100 dark:bg-green-900/20 flex items-center justify-center",
                            Icon { icon: LdPlus, width: 18, height: 18, fill: "currentColor", class: "text-green-600 dark:text-green-400" }
                        }
                        div {
                            p { class: "text-sm font-semibold", "New Post" }
                            p { class: "text-xs text-muted-foreground", "Create a new blog post" }
                        }
                    }
                    div {
                        class: "flex items-center gap-3 p-4 rounded-lg border border-border/70 bg-card hover:bg-accent/50 cursor-pointer transition-colors",
                        onclick: move |_| { nav.push(Route::MediaUploadScreen {}); },
                        div { class: "w-10 h-10 rounded-full bg-blue-100 dark:bg-blue-900/20 flex items-center justify-center",
                            Icon { icon: LdUpload, width: 18, height: 18, fill: "currentColor", class: "text-blue-600 dark:text-blue-400" }
                        }
                        div {
                            p { class: "text-sm font-semibold", "Upload Media" }
                            p { class: "text-xs text-muted-foreground", "Add images and files" }
                        }
                    }
                    div {
                        class: "flex items-center gap-3 p-4 rounded-lg border border-border/70 bg-card hover:bg-accent/50 cursor-pointer transition-colors",
                        onclick: move |_| { nav.push(Route::HomeScreen {}); },
                        div { class: "w-10 h-10 rounded-full bg-purple-100 dark:bg-purple-900/20 flex items-center justify-center",
                            Icon { icon: LdSend, width: 18, height: 18, fill: "currentColor", class: "text-purple-600 dark:text-purple-400" }
                        }
                        div {
                            p { class: "text-sm font-semibold", "Send Newsletter" }
                            p { class: "text-xs text-muted-foreground", "Email your subscribers" }
                        }
                    }
                }

                // Recent Comments widget
                div { class: "rounded-lg border border-border/70 bg-card p-6 mb-8",
                    h2 { class: "text-lg font-semibold mb-4", "Recent Comments" }
                    div { class: "divide-y divide-border",
                        for comment in recent_comments.iter() {
                            {
                                let author = comment.author.clone();
                                let post_title = comment.post_title.clone();
                                let date = comment.date.clone();
                                rsx! {
                                    div { class: "flex items-start justify-between py-3 first:pt-0 last:pb-0",
                                        div { class: "space-y-1",
                                            p { class: "text-sm font-medium", "{author}" }
                                            p { class: "text-xs text-muted-foreground",
                                                "on "
                                                span { class: "font-medium text-foreground", "{post_title}" }
                                            }
                                        }
                                        span { class: "text-xs text-muted-foreground shrink-0 ml-4", "{date}" }
                                    }
                                }
                            }
                        }
                    }
                }

                // Getting started section
                div { class: "rounded-lg border border-border/70 bg-card p-6",
                    h2 { class: "text-xl font-semibold mb-4", "Getting Started" }
                    div { class: "space-y-3",
                        div { class: "flex items-start gap-3",
                            div { class: "w-6 h-6 rounded-full bg-primary text-primary-foreground flex items-center justify-center text-sm font-medium shrink-0", "1" }
                            div {
                                h3 { class: "font-medium mb-1", "Create your first post" }
                                p { class: "text-sm text-muted-foreground", "Start writing content for your blog" }
                            }
                        }
                        div { class: "flex items-start gap-3",
                            div { class: "w-6 h-6 rounded-full bg-primary text-primary-foreground flex items-center justify-center text-sm font-medium shrink-0", "2" }
                            div {
                                h3 { class: "font-medium mb-1", "Organize with categories" }
                                p { class: "text-sm text-muted-foreground", "Create categories to organize your content" }
                            }
                        }
                        div { class: "flex items-start gap-3",
                            div { class: "w-6 h-6 rounded-full bg-primary text-primary-foreground flex items-center justify-center text-sm font-medium shrink-0", "3" }
                            div {
                                h3 { class: "font-medium mb-1", "Add tags for discoverability" }
                                p { class: "text-sm text-muted-foreground", "Help readers find related content" }
                            }
                        }
                        div { class: "flex items-start gap-3",
                            div { class: "w-6 h-6 rounded-full bg-primary text-primary-foreground flex items-center justify-center text-sm font-medium shrink-0", "4" }
                            div {
                                h3 { class: "font-medium mb-1", "Upload media" }
                                p { class: "text-sm text-muted-foreground", "Add images to make your posts more engaging" }
                            }
                        }
                    }
                }
            }
        }
    }
}

// Full analytics dashboard
#[cfg(feature = "analytics")]
fn analytics_dashboard() -> Element {
    use crate::router::Route;
    use hmziq_dioxus_free_icons::{icons::ld_icons::*, Icon};

    let nav = use_navigator();
    let analytics = use_analytics();
    let filters = use_analytics_filters();
    let recent_comments = mock_recent_comments();

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

            // NOTE: Commented out overkill analytics fetches for personal blog
            // Registration trends
            // let registration_req = RegistrationTrendsRequest {
            //     envelope: envelope.clone(),
            //     filters: RegistrationTrendsFilters {
            //         group_by: AnalyticsInterval::Day,
            //     },
            // };
            // analytics.fetch_registration_trends(registration_req).await;

            // Verification rates
            // let verification_req = VerificationRatesRequest {
            //     envelope: envelope.clone(),
            //     filters: VerificationRatesFilters {
            //         group_by: AnalyticsInterval::Day,
            //     },
            // };
            // analytics.fetch_verification_rates(verification_req).await;
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
    // NOTE: Commented out overkill analytics frames for personal blog
    // let registration_frame = analytics.registration_trends.read();
    // let verification_frame = analytics.verification_rates.read();

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

                // Quick Actions section
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                    div {
                        class: "flex items-center gap-3 p-4 rounded-lg border border-border/70 bg-card hover:bg-accent/50 cursor-pointer transition-colors",
                        onclick: move |_| { nav.push(Route::PostsAddScreen {}); },
                        div { class: "w-10 h-10 rounded-full bg-green-100 dark:bg-green-900/20 flex items-center justify-center",
                            Icon { icon: LdPlus, width: 18, height: 18, fill: "currentColor", class: "text-green-600 dark:text-green-400" }
                        }
                        div {
                            p { class: "text-sm font-semibold", "New Post" }
                            p { class: "text-xs text-muted-foreground", "Create a new blog post" }
                        }
                    }
                    div {
                        class: "flex items-center gap-3 p-4 rounded-lg border border-border/70 bg-card hover:bg-accent/50 cursor-pointer transition-colors",
                        onclick: move |_| { nav.push(Route::MediaUploadScreen {}); },
                        div { class: "w-10 h-10 rounded-full bg-blue-100 dark:bg-blue-900/20 flex items-center justify-center",
                            Icon { icon: LdUpload, width: 18, height: 18, fill: "currentColor", class: "text-blue-600 dark:text-blue-400" }
                        }
                        div {
                            p { class: "text-sm font-semibold", "Upload Media" }
                            p { class: "text-xs text-muted-foreground", "Add images and files" }
                        }
                    }
                    div {
                        class: "flex items-center gap-3 p-4 rounded-lg border border-border/70 bg-card hover:bg-accent/50 cursor-pointer transition-colors",
                        onclick: move |_| { nav.push(Route::HomeScreen {}); },
                        div { class: "w-10 h-10 rounded-full bg-purple-100 dark:bg-purple-900/20 flex items-center justify-center",
                            Icon { icon: LdSend, width: 18, height: 18, fill: "currentColor", class: "text-purple-600 dark:text-purple-400" }
                        }
                        div {
                            p { class: "text-sm font-semibold", "Send Newsletter" }
                            p { class: "text-xs text-muted-foreground", "Email your subscribers" }
                        }
                    }
                }

                // Recent Comments widget
                div { class: "rounded-lg border border-border/70 bg-card p-6",
                    h2 { class: "text-lg font-semibold mb-4", "Recent Comments" }
                    div { class: "divide-y divide-border",
                        for comment in recent_comments.iter() {
                            {
                                let author = comment.author.clone();
                                let post_title = comment.post_title.clone();
                                let date = comment.date.clone();
                                rsx! {
                                    div { class: "flex items-start justify-between py-3 first:pt-0 last:pb-0",
                                        div { class: "space-y-1",
                                            p { class: "text-sm font-medium", "{author}" }
                                            p { class: "text-xs text-muted-foreground",
                                                "on "
                                                span { class: "font-medium text-foreground", "{post_title}" }
                                            }
                                        }
                                        span { class: "text-xs text-muted-foreground shrink-0 ml-4", "{date}" }
                                    }
                                }
                            }
                        }
                    }
                }

                // NOTE: Commented out overkill analytics UI for personal blog
                // Secondary charts row: registrations & verifications
                // div { class: "grid grid-cols-1 lg:grid-cols-2 gap-4",
                //     RegistrationTrendChart {
                //         frame: registration_frame.clone(),
                //         title: "New user registrations (last 7 days)".to_string(),
                //         height: "h-64".to_string(),
                //     }
                //
                //     VerificationRatesChart {
                //         frame: verification_frame.clone(),
                //         title: "Verification funnel".to_string(),
                //         height: "260px".to_string(),
                //         show_success_rate: true,
                //         on_interval_change: Some(EventHandler::new(move |interval: AnalyticsInterval| {
                //             spawn(async move {
                //                 let envelope = filters.build_envelope();
                //                 let req = VerificationRatesRequest {
                //                     envelope,
                //                     filters: VerificationRatesFilters {
                //                         group_by: interval,
                //                     },
                //                 };
                //                 analytics.fetch_verification_rates(req).await;
                //             });
                //         })),
                //     }
                // }
            }
        }
    }
}
