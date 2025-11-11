use dioxus::prelude::*;

use crate::components::{ErrorDetails, ErrorDetailsVariant, LoadingOverlay};
use crate::store::{
    use_analytics, AnalyticsEnvelope, AnalyticsEnvelopeResponse, AnalyticsInterval,
    NewsletterGrowthFilters, NewsletterGrowthPoint, NewsletterGrowthRequest, StateFrameStatus,
};

/// Props for `NewsletterGrowthChart`.
/// Minimal: takes a title and height override if you want to embed it differently.
#[derive(Props, Clone, PartialEq)]
pub struct NewsletterGrowthChartProps {
    #[props(default = "Newsletter growth".to_string())]
    pub title: String,
    /// Tailwind-friendly height class for the chart container.
    #[props(default = "h-80".to_string())]
    pub height_class: String,
    /// Default interval for initial fetch.
    #[props(default = AnalyticsInterval::Week)]
    pub default_interval: AnalyticsInterval,
}

/// High-level chart card visualizing newsletter growth over time.
///
/// This is intentionally scaffolded:
/// - Uses `use_analytics().newsletter_growth` signal and `fetch_newsletter_growth` action.
/// - Renders a styled card shell wired to `StateFrame` lifecycle:
///   - Loading skeleton
///   - Error banner surfaced via toast
///   - Empty state
///   - Data view with a simple SVG-based chart placeholder
///
/// Once `dioxus-charts` is added, the SVG block can be replaced with real chart primitives.
#[component]
pub fn NewsletterGrowthChart(props: NewsletterGrowthChartProps) -> Element {
    let analytics = use_analytics();

    // Local interval state (for future filter controls).
    let interval = use_signal(|| props.default_interval.clone());

    // One-time initial fetch on mount.
    // In future iterations we can refetch when `interval` changes.
    use_future(move || {
        let analytics = analytics;
        let interval = interval.read().clone();

        async move {
            // Map enum to backend interval if needed (we pass the enum directly below).

            let request = NewsletterGrowthRequest {
                envelope: AnalyticsEnvelope {
                    date_from: None,
                    date_to: None,
                    page: None,
                    per_page: None,
                    sort_by: None,
                    sort_order: None,
                },
                filters: NewsletterGrowthFilters {
                    group_by: match interval {
                        AnalyticsInterval::Hour => AnalyticsInterval::Hour,
                        AnalyticsInterval::Day => AnalyticsInterval::Day,
                        AnalyticsInterval::Week => AnalyticsInterval::Week,
                        AnalyticsInterval::Month => AnalyticsInterval::Month,
                    },
                },
            };

            analytics.fetch_newsletter_growth(request).await;
        }
    });

    let frame = analytics.newsletter_growth.read();

    let status = frame.status;
    let error = frame.error.clone();
    let data = frame.data.clone();
    let is_loading = status == StateFrameStatus::Loading;

    let (status_str, body) = if status == StateFrameStatus::Loading {
        (
            "Loading",
            rsx! {
                LoadingSkeleton { }
            },
        )
    } else if status == StateFrameStatus::Failed {
        (
            "Error",
            rsx! {
                ErrorDetails {
                    error: error.clone(),
                    variant: ErrorDetailsVariant::Collapsed,
                    title: Some("Failed to load newsletter growth data".to_string()),
                }
            },
        )
    } else if status == StateFrameStatus::Success {
        if let Some(AnalyticsEnvelopeResponse { data, .. }) = data {
            if data.is_empty() {
                (
                    "Empty",
                    rsx! {
                        EmptyState {
                            message: "No newsletter activity in the selected period yet.".to_string(),
                        }
                    },
                )
            } else {
                (
                    "Ready",
                    rsx! {
                        NewsletterGrowthChartInner { points: data.clone() }
                    },
                )
            }
        } else {
            (
                "Idle",
                rsx! {
                    EmptyState {
                        message: "Newsletter growth data will appear here once available.".to_string(),
                    }
                },
            )
        }
    } else {
        (
            "Idle",
            rsx! {
                EmptyState {
                    message: "Newsletter growth data will appear here once available.".to_string(),
                }
            },
        )
    };

    rsx! {
        div {
            class: "rounded-2xl border border-border bg-background shadow-none flex flex-col gap-3 p-4 {props.height_class} relative",
            // Header
            div {
                class: "flex items-center justify-between gap-2",
                h3 {
                    class: "text-sm font-semibold text-zinc-900 dark:text-zinc-100",
                    "{props.title}"
                }
                span {
                    class: "text-[10px] px-2 py-0.5 rounded-full bg-background border border-border text-muted-foreground",
                    "{status_str}"
                }
            }

            // Placeholder controls row (interval etc.) - wired later
            div {
                class: "flex items-center justify-between gap-2 text-[10px] text-zinc-500",
                span { "Stacked bars: new / confirmed / unsubscribed, line: net growth." }
                // Future: interval selector, etc.
            }

            // Body
            div {
                class: "flex-1 mt-1",
                {body}
            }

            LoadingOverlay { visible: is_loading }
        }
    }
}

#[component]
fn LoadingSkeleton() -> Element {
    rsx! {
        div {
            class: "w-full h-full flex flex-col justify-between animate-pulse",
            div { class: "h-4 w-24 bg-muted rounded-md mb-2" }
            div { class: "flex-1 flex items-end gap-1",
                {(0..10).map(|i| {
                    let h = 20 + (i * 4);
                    rsx! {
                        div {
                            key: "{i}",
                            class: "flex-1 bg-muted rounded-t-md",
                            style: "height: {h}px;",
                        }
                    }
                })}
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct EmptyStateProps {
    message: String,
}

#[component]
fn EmptyState(props: EmptyStateProps) -> Element {
    rsx! {
        div {
            class: "w-full h-full flex items-center justify-center",
            span {
                class: "text-[11px] text-zinc-500 dark:text-zinc-500",
                "{props.message}"
            }
        }
    }
}

/// Inner chart presentation using basic SVG as a placeholder.
/// This keeps the data mapping explicit and can be swapped for dioxus-charts later
/// without changing the public `NewsletterGrowthChart` API.
#[derive(Props, Clone, PartialEq)]
struct NewsletterGrowthChartInnerProps {
    points: Vec<NewsletterGrowthPoint>,
}

#[component]
fn NewsletterGrowthChartInner(props: NewsletterGrowthChartInnerProps) -> Element {
    let points = &props.points;

    // Compute basic ranges for crude scaling.
    let mut max_bar = 0_i64;
    let mut max_net = 0_i64;
    let mut min_net = 0_i64;

    for p in points.iter() {
        let bar_total = p.new_subscribers.max(0) + p.confirmed.max(0) + p.unsubscribed.max(0);
        if bar_total > max_bar {
            max_bar = bar_total;
        }
        if p.net_growth > max_net {
            max_net = p.net_growth;
        }
        if p.net_growth < min_net {
            min_net = p.net_growth;
        }
    }

    let _max_bar = max_bar.max(1);
    let max_abs_net = max_net.abs().max(min_net.abs()).max(1);

    let width = 1000.0;
    let height = 220.0;
    let padding_left = 28.0;
    let padding_right = 8.0;
    let padding_top = 10.0;
    let padding_bottom = 22.0;

    let chart_width = f64::max(width - padding_left - padding_right, 1.0);
    let chart_height = f64::max(height - padding_top - padding_bottom, 1.0);

    let n = points.len().max(1);
    let step = chart_width / n as f64;

    rsx! {
        div { class: "w-full h-full flex flex-col gap-1",
            // Legend
            div {
                class: "flex flex-wrap items-center gap-3 text-[9px] text-muted-foreground",
                LegendDot { class_name: "bg-emerald-500/90" }
                span { "New subscribers" }
                LegendDot { class_name: "bg-sky-500/90" }
                span { "Confirmed" }
                LegendDot { class_name: "bg-rose-400/90" }
                span { "Unsubscribed" }
                LegendDot { class_name: "bg-amber-400/90" }
                span { "Net growth (line)" }
            }

            // Simple SVG chart placeholder
            div { class: "relative flex-1 mt-1",
                svg {
                    class: "w-full h-full text-[8px]",
                    view_box: "0 0 {width} {height}",

                    // Background
                    rect {
                        x: "0",
                        y: "0",
                        width: "{width}",
                        height: "{height}",
                        fill: "transparent",
                    }

                    // Horizontal grid lines (TODO: restore dynamic generation)
                    line { x1: "{padding_left}", x2: "{width - padding_right}", y1: "{padding_top}", y2: "{padding_top}", stroke: "currentColor", class: "text-zinc-200/80 dark:text-zinc-800/80", "stroke-width": "0.6" }
                    line { x1: "{padding_left}", x2: "{width - padding_right}", y1: "{padding_top + chart_height}", y2: "{padding_top + chart_height}", stroke: "currentColor", class: "text-zinc-200/80 dark:text-zinc-800/80", "stroke-width": "0.6" }

                    // Bars placeholder (TODO: restore full chart rendering)
                    text {
                        x: "{width / 2.0}",
                        y: "{height / 2.0}",
                        text_anchor: "middle",
                        class: "fill-zinc-400 text-[10px]",
                        "Chart: {points.len()} data points"
                    }

                    {
                        // Net growth polyline drawn after bars.
                        if points.len() >= 2 {
                            let d = net_points_to_path(&points, padding_left, step, padding_top, chart_height, max_abs_net);
                            rsx! {
                                path {
                                    d: "{d}",
                                    fill: "none",
                                    class: "stroke-amber-400/90",
                                    "stroke-width": "2",
                                }
                            }
                        } else if points.len() == 1 {
                            // Single point: draw a small circle
                            let (x, y) = single_net_point(&points, padding_left, step, padding_top, chart_height, max_abs_net);
                            rsx! {
                                circle {
                                    cx: "{x}",
                                    cy: "{y}",
                                    r: "3",
                                    class: "fill-amber-400/90",
                                }
                            }
                        } else {
                            rsx! {}
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn LegendDot(class_name: String) -> Element {
    rsx! {
        span {
            class: "inline-block w-2 h-2 rounded-full {class_name}",
        }
    }
}

// Helpers kept private to this module.

fn _truncate_label(label: &str, max_len: usize) -> String {
    if label.chars().count() <= max_len {
        label.to_string()
    } else {
        let mut s = label
            .chars()
            .take(max_len.saturating_sub(1))
            .collect::<String>();
        s.push('…');
        s
    }
}

fn net_points_to_path(
    points: &[NewsletterGrowthPoint],
    padding_left: f64,
    step: f64,
    padding_top: f64,
    chart_height: f64,
    max_abs_net: i64,
) -> String {
    if points.is_empty() {
        return String::new();
    }

    let mut d = String::new();
    for (i, p) in points.iter().enumerate() {
        let x = padding_left + step * (i as f64 + 0.5);
        let net_ratio = p.net_growth as f64 / max_abs_net as f64;
        let y = padding_top + (chart_height * (1.0 - (net_ratio + 1.0) / 2.0));

        if i == 0 {
            d.push_str(&format!("M{:.2},{:.2}", x, y));
        } else {
            d.push_str(&format!(" L{:.2},{:.2}", x, y));
        }
    }
    d
}

fn single_net_point(
    points: &[NewsletterGrowthPoint],
    padding_left: f64,
    step: f64,
    padding_top: f64,
    chart_height: f64,
    max_abs_net: i64,
) -> (f64, f64) {
    let p = &points[0];
    let x = padding_left + step * 0.5;
    let net_ratio = p.net_growth as f64 / max_abs_net as f64;
    let y = padding_top + (chart_height * (1.0 - (net_ratio + 1.0) / 2.0));
    (x, y)
}
