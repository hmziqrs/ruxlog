use dioxus::prelude::*;

use crate::store::analytics::{AnalyticsEnvelopeResponse, PublishingTrendPoint};
use crate::store::StateFrame;

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
        crate::store::PublishingTrendsRequest,
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
            class: "rounded-2xl border border-border bg-background shadow-none flex flex-col",

            // Header
            div {
                class: "flex items-center justify-between px-4 pt-3 pb-2 gap-2",
                div {
                    class: "flex flex-col",
                    h2 {
                        class: "text-sm font-semibold text-zinc-900 dark:text-zinc-50 truncate",
                        "{title}"
                    }
                    if let Some(desc) = description {
                        p {
                            class: "text-[10px] text-zinc-500 dark:text-zinc-400 truncate",
                            "{desc}"
                        }
                    } else {
                        p {
                            class: "text-[10px] text-zinc-500 dark:text-zinc-400",
                            "Stacked view of posts by status over time."
                        }
                    }
                }

                // Simple legend scaffold (colors/statuses to align with future stacked series)
                div {
                    class: "flex items-center gap-2 text-[9px] text-zinc-500 dark:text-zinc-400",
                    LegendDot { class_name: "bg-emerald-500/80" }
                    span { "Published" }
                    LegendDot { class_name: "bg-sky-500/80" }
                    span { "Draft" }
                    LegendDot { class_name: "bg-amber-500/80" }
                    span { "Scheduled" }
                }
            }

            // Body: different states
            div {
                class: format!("relative px-3 pb-3 {}", height_class),

                // Loading state: skeleton bars
                if loading {
                    div {
                        class: "absolute inset-0 flex items-end justify-between gap-1 px-1",
                        for i in 0..10 {
                            {
                                let h = 20 + (i * 5) % 60;
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
                        class: "flex flex-col items-start justify-center gap-1 h-full \
                                rounded-xl border border-destructive bg-background px-3 py-2",
                        span {
                            class: "text-[11px] font-semibold text-destructive",
                            "Unable to load publishing trends"
                        }
                        if let Some(msg) = error {
                            span {
                                class: "text-[10px] text-rose-600/90 dark:text-rose-400/90 line-clamp-3",
                                "{msg}"
                            }
                        } else {
                            span {
                                class: "text-[10px] text-rose-600/90 dark:text-rose-400/90",
                                "Try adjusting filters or reloading the dashboard."
                            }
                        }
                    }
                // Empty state
                } else if is_empty {
                    div {
                        class: "flex flex-col items-center justify-center gap-1 h-full \
                                rounded-xl border border-dashed border-zinc-200/70 dark:border-zinc-800/70 \
                                text-[10px] text-zinc-500 dark:text-zinc-400 px-3",
                        span { "No publishing activity found for the selected period." }
                        span { "Publish new posts or widen your date range to see trends here." }
                    }
                // Data state (temporary textual visualization)
                } else {
                    // Until `dioxus-charts` is wired, show a simple proportional bar layout
                    // so the component is still informative and testable.
                    {
                        let max_total = data
                            .iter()
                            .map(|p| p.counts.values().sum::<i64>())
                            .max()
                            .unwrap_or(1)
                            .max(1) as f64;

                        rsx! { div {
                        class: "flex flex-col gap-1 h-full",

                        // Buckets row (scrollable for many buckets)
                        div {
                            class: "flex-1 flex items-end gap-1 overflow-x-auto overflow-y-visible pb-1",
                            { data.iter().map(|point| {
                                let total: i64 = point.counts.values().sum();
                                let height_pct = ((total as f64 / max_total) * 100.0).max(8.0);

                                // Extract status buckets with stable ordering.
                                let published = *point.counts.get("published").unwrap_or(&0);
                                let draft = *point.counts.get("draft").unwrap_or(&0);
                                let scheduled = *point.counts.get("scheduled").unwrap_or(&0);
                                let other_total = total - (published + draft + scheduled);

                                rsx! {
                                    div {
                                        key: "{point.bucket}",
                                        class: "flex flex-col items-center gap-0.5 min-w-[32px]",
                                        // Column label (bucket)
                                        span {
                                            class: "text-[8px] text-zinc-500 dark:text-zinc-500 truncate max-w-[40px]",
                                            "{point.bucket}"
                                        }
                                        // Stacked pseudo-bar
                                        div {
                                            class: "w-full rounded-t-md overflow-hidden flex flex-col-reverse \
                                                    bg-background border border-border",
                                            style: "height: {height_pct}%; min-height: 28px;",

                                            if total > 0 {
                                                if other_total > 0 {
                                                    div {
                                                        class: "w-full bg-muted",
                                                        style: format!("height: {}%;", (other_total as f64 / total as f64) * 100.0),
                                                    }
                                                }
                                                if scheduled > 0 {
                                                    div {
                                                        class: "w-full bg-amber-500/80",
                                                        style: format!("height: {}%;", (scheduled as f64 / total as f64) * 100.0),
                                                    }
                                                }
                                                if draft > 0 {
                                                    div {
                                                        class: "w-full bg-sky-500/80",
                                                        style: format!("height: {}%;", (draft as f64 / total as f64) * 100.0),
                                                    }
                                                }
                                                if published > 0 {
                                                    div {
                                                        class: "w-full bg-emerald-500/90",
                                                        style: format!("height: {}%;", (published as f64 / total as f64) * 100.0),
                                                    }
                                                }
                                            }
                                        }
                                        // Total label
                                        span {
                                            class: "text-[8px] text-zinc-600 dark:text-zinc-400 font-medium",
                                            "{total}"
                                        }
                                    }
                                }
                            })}
                        }

                        // Meta hint row
                        div {
                            class: "flex items-center justify-between text-[8px] text-zinc-500 dark:text-zinc-500 mt-0.5",
                            span {
                                "Each column shows total posts by status per interval bucket."
                            }
                            span {
                                class: "hidden xl:inline",
                                "Upgraded to full dioxus-charts stacked bars in the next step."
                            }
                        }
                    }}
                    }
                }
            }
        }
    }
}

/// Small legend dot used in the header.
#[component]
fn LegendDot(class_name: &'static str) -> Element {
    rsx! {
        span {
            class: format!(
                "w-2 h-2 rounded-full inline-block border border-zinc-950/5 {}",
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
fn from_state_frame(
    title: String,
    frame: &StateFrame<
        AnalyticsEnvelopeResponse<Vec<PublishingTrendPoint>>,
        crate::store::PublishingTrendsRequest,
    >,
) -> PublishingTrendsChartProps {
    PublishingTrendsChartProps {
        frame: frame.clone(),
        title,
        height_class: "h-72".to_string(),
        description: Some("Posts by status across the selected interval.".to_string()),
    }
}
