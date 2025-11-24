use dioxus::prelude::*;
use oxstore::StateFrame;

use ruxlog_shared::store::analytics::{AnalyticsEnvelopeResponse, PublishingTrendPoint};

/// Props for `PublishingTrendsChart`.
///
/// This component is intentionally minimal and focused:
/// - Receives a `StateFrame` for publishing trends data.
/// - Does not own/filter state itself; higher-level screens manage requests.
/// - Extracts data, loading, and error from the frame internally.
#[derive(Props, PartialEq, Clone)]
pub struct PublishingTrendsChartProps {
    /// State frame containing publishing trend points.
    pub frame: StateFrame<
        AnalyticsEnvelopeResponse<Vec<PublishingTrendPoint>>,
        ruxlog_shared::store::PublishingTrendsRequest,
    >,

    /// Chart title shown in the card header.
    #[props(default = "Publishing Trends".to_string())]
    pub title: String,

    /// Optional explicit height (Tailwind class). Defaults to `h-72`.
    #[props(default = "h-72".to_string())]
    pub height_class: String,

    /// Optional description or subtitle beneath the title.
    #[props(default)]
    pub description: Option<String>,
}

/// Card wrapper for stacked publishing trends bar chart.
///
/// NOTE:
/// - This file intentionally does NOT depend on `dioxus-charts` directly yet.
///   That integration will be added once the dependency is wired up and chart
///   primitives are confirmed. For now, it exposes a clean shell with clear data
///   mapping and states (loading/empty/error) so it can be quickly upgraded.
///
/// Expected future mapping (with `dioxus-charts`):
/// - X axis: `bucket`
/// - Y axis: counts per status from `counts` map (stacked bars).
#[component]
pub fn PublishingTrendsChart(props: PublishingTrendsChartProps) -> Element {
    let PublishingTrendsChartProps {
        frame,
        title,
        height_class,
        description,
    } = props;

    // Extract state from frame.
    let loading = frame.is_loading();
    let has_error = frame.error.is_some();
    let error = frame.error_message();
    let data = frame
        .data
        .as_ref()
        .map(|env| env.data.clone())
        .unwrap_or_default();
    let is_empty = !loading && !has_error && data.is_empty();

    rsx! {
        div {
            class: "rounded-xl border border-border bg-card flex flex-col",

            // Header
            div {
                class: "flex items-center justify-between px-4 pt-4 pb-2 gap-2",
                div {
                    class: "flex flex-col gap-0.5",
                    h2 {
                        class: "text-sm font-semibold text-foreground",
                        "{title}"
                    }
                    if let Some(desc) = description {
                        p {
                            class: "text-xs text-muted-foreground",
                            "{desc}"
                        }
                    } else {
                        p {
                            class: "text-xs text-muted-foreground",
                            "Posts by status over time"
                        }
                    }
                }

                // Simplified legend for personal blog (just Published/Draft)
                div {
                    class: "flex items-center gap-3 text-xs text-muted-foreground",
                    div { class: "flex items-center gap-1.5",
                        span { class: "w-2.5 h-2.5 rounded-sm bg-emerald-500" }
                        span { "Published" }
                    }
                    div { class: "flex items-center gap-1.5",
                        span { class: "w-2.5 h-2.5 rounded-sm bg-sky-500" }
                        span { "Draft" }
                    }
                }
            }

            // Body: different states
            div {
                class: format!("relative px-4 pb-4 {}", height_class),

                // Loading state: skeleton bars
                if loading {
                    div {
                        class: "absolute inset-0 flex items-end justify-around gap-2 px-4 pb-4",
                        for i in 0..8 {
                            {
                                let h = 30 + (i * 7) % 50;
                                rsx! {
                                    div {
                                        key: "{i}",
                                        class: "flex-1 flex items-end",
                                        div {
                                            class: "w-full rounded-t bg-muted animate-pulse",
                                            style: "height: {h}%;",
                                        }
                                    }
                                }
                            }
                        }
                    }
                // Error state
                } else if has_error {
                    div {
                        class: "flex flex-col items-center justify-center gap-2 h-full",
                        div {
                            class: "w-10 h-10 rounded-full bg-destructive/10 flex items-center justify-center",
                            svg {
                                class: "w-5 h-5 text-destructive",
                                xmlns: "http://www.w3.org/2000/svg",
                                fill: "none",
                                view_box: "0 0 24 24",
                                stroke_width: "1.5",
                                stroke: "currentColor",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    d: "M12 9v3.75m9-.75a9 9 0 1 1-18 0 9 9 0 0 1 18 0Zm-9 3.75h.008v.008H12v-.008Z"
                                }
                            }
                        }
                        span {
                            class: "text-sm font-medium text-foreground",
                            "Unable to load publishing trends"
                        }
                        if let Some(msg) = error {
                            span {
                                class: "text-xs text-muted-foreground text-center max-w-xs",
                                "{msg}"
                            }
                        }
                    }
                // Empty state
                } else if is_empty {
                    div {
                        class: "flex flex-col items-center justify-center gap-2 h-full",
                        div {
                            class: "w-10 h-10 rounded-full bg-muted flex items-center justify-center",
                            svg {
                                class: "w-5 h-5 text-muted-foreground",
                                xmlns: "http://www.w3.org/2000/svg",
                                fill: "none",
                                view_box: "0 0 24 24",
                                stroke_width: "1.5",
                                stroke: "currentColor",
                                path {
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    d: "M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z"
                                }
                            }
                        }
                        span { class: "text-sm font-medium text-foreground", "No posts in this period" }
                        span { class: "text-xs text-muted-foreground text-center max-w-xs", "Start writing to see your publishing activity here." }
                    }
                // Data state - clean bar visualization
                } else {
                    {
                        let max_total = data
                            .iter()
                            .map(|p| p.counts.values().sum::<i64>())
                            .max()
                            .unwrap_or(1)
                            .max(1) as f64;

                        // Calculate total for summary
                        let total_published: i64 = data.iter()
                            .map(|p| *p.counts.get("Published").or(p.counts.get("published")).unwrap_or(&0))
                            .sum();
                        let total_drafts: i64 = data.iter()
                            .map(|p| *p.counts.get("Draft").or(p.counts.get("draft")).unwrap_or(&0))
                            .sum();

                        rsx! { div {
                        class: "flex flex-col h-full",

                        // Buckets row (scrollable for many buckets)
                        div {
                            class: "flex-1 flex items-end gap-2 overflow-x-auto pb-2",
                            { data.iter().map(|point| {
                                let total: i64 = point.counts.values().sum();
                                let height_pct = ((total as f64 / max_total) * 100.0).max(12.0);

                                // Extract status buckets - handle both capitalized and lowercase keys
                                let published = *point.counts.get("Published")
                                    .or(point.counts.get("published"))
                                    .unwrap_or(&0);
                                let draft = *point.counts.get("Draft")
                                    .or(point.counts.get("draft"))
                                    .unwrap_or(&0);

                                rsx! {
                                    div {
                                        key: "{point.bucket}",
                                        class: "flex flex-col items-center gap-1 min-w-[36px] group",
                                        // Stacked bar
                                        div {
                                            class: "w-full rounded-t overflow-hidden flex flex-col-reverse \
                                                    bg-muted/30 transition-all group-hover:bg-muted/50",
                                            style: "height: {height_pct}%; min-height: 24px;",

                                            if total > 0 {
                                                if draft > 0 {
                                                    div {
                                                        class: "w-full bg-sky-500 transition-colors",
                                                        style: format!("height: {}%;", (draft as f64 / total as f64) * 100.0),
                                                    }
                                                }
                                                if published > 0 {
                                                    div {
                                                        class: "w-full bg-emerald-500 transition-colors",
                                                        style: format!("height: {}%;", (published as f64 / total as f64) * 100.0),
                                                    }
                                                }
                                            }
                                        }
                                        // Column label (bucket) - format nicely
                                        span {
                                            class: "text-[10px] text-muted-foreground truncate max-w-[48px]",
                                            "{format_bucket_label(&point.bucket)}"
                                        }
                                    }
                                }
                            })}
                        }

                        // Summary row
                        div {
                            class: "flex items-center justify-between pt-2 border-t border-border text-xs",
                            div { class: "flex items-center gap-4 text-muted-foreground",
                                span {
                                    "Total: "
                                    span { class: "font-medium text-foreground", "{total_published + total_drafts}" }
                                    " posts"
                                }
                            }
                            div { class: "flex items-center gap-3 text-muted-foreground",
                                span {
                                    span { class: "text-emerald-500 font-medium", "{total_published}" }
                                    " published"
                                }
                                span {
                                    span { class: "text-sky-500 font-medium", "{total_drafts}" }
                                    " drafts"
                                }
                            }
                        }
                    }}
                    }
                }
            }
        }
    }
}

/// Format bucket label for cleaner display
fn format_bucket_label(bucket: &str) -> String {
    // Handle week format "2024-W01" -> "W01"
    if bucket.contains("-W") {
        return bucket.split('-').last().unwrap_or(bucket).to_string();
    }
    // Handle month format "2024-01" -> "Jan"
    if bucket.len() == 7 && bucket.chars().nth(4) == Some('-') {
        let month = bucket.split('-').last().unwrap_or("01");
        return match month {
            "01" => "Jan", "02" => "Feb", "03" => "Mar", "04" => "Apr",
            "05" => "May", "06" => "Jun", "07" => "Jul", "08" => "Aug",
            "09" => "Sep", "10" => "Oct", "11" => "Nov", "12" => "Dec",
            _ => bucket,
        }.to_string();
    }
    // Handle day format "2024-01-15" -> "15"
    if bucket.len() == 10 {
        return bucket.split('-').last().unwrap_or(bucket).to_string();
    }
    bucket.to_string()
}

/// Small legend dot used in the header.
#[allow(dead_code)]
#[component]
fn LegendDot(class_name: &'static str) -> Element {
    rsx! {
        span {
            class: format!(
                "w-2 h-2 rounded-full inline-block {}",
                class_name
            ),
        }
    }
}

/// Helper to map a `StateFrame` of publishing trends into `PublishingTrendsChart` props.
///
/// This is optional, but gives screens a convenient way to bind store state.
/// It does not couple the chart to any specific request type.
///
/// Usage from a screen (conceptual):
///
/// ```ignore
/// let analytics = use_analytics();
/// let frame = analytics.publishing_trends.read();
///
/// rsx! {
///     PublishingTrendsChart { frame: frame.clone(), title: "Publishing Trends".into() }
/// }
/// ```
#[allow(dead_code)]
fn from_state_frame(
    title: String,
    frame: &StateFrame<
        AnalyticsEnvelopeResponse<Vec<PublishingTrendPoint>>,
        ruxlog_shared::store::PublishingTrendsRequest,
    >,
) -> PublishingTrendsChartProps {
    PublishingTrendsChartProps {
        frame: frame.clone(),
        title,
        height_class: "h-72".to_string(),
        description: Some("Posts by status across the selected interval.".to_string()),
    }
}
