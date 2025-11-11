use dioxus::prelude::*;

use crate::components::{ErrorDetails, ErrorDetailsVariant, LoadingOverlay};
use crate::store::{use_analytics, CommentRatePoint, StateFrameStatus};

/// Props for `CommentRateChart`.
///
/// - `points`: Data points to visualize, typically from analytics state.
/// - `title`: Optional title rendered in the card header.
/// - `height`: Optional explicit height class (e.g. "h-80"). Defaults to "h-80".
#[derive(Props, PartialEq, Clone)]
pub struct CommentRateChartProps {
    pub points: Vec<CommentRatePoint>,
    #[props(optional)]
    pub title: Option<String>,
    #[props(optional)]
    pub height: Option<String>,
}

/// Simple horizontal bar chart component for comment rate analytics.
///
/// This is intentionally scaffolded without external chart dependencies so it can
/// be wired immediately and later upgraded to `dioxus-charts` primitives.
///
/// Recommended usage:
/// - Use on the Analytics screen with data from:
///   `use_analytics().comment_rate`'s `StateFrame`
/// - Show top N posts by `comment_rate` or `comments` at the caller level.
#[component]
pub fn CommentRateChart(props: CommentRateChartProps) -> Element {
    let title = props
        .title
        .clone()
        .unwrap_or_else(|| "Top posts by comment rate".to_string());
    let height_class = props.height.clone().unwrap_or_else(|| "h-80".to_string());

    // Determine max value for relative width scaling.
    let max_value = props
        .points
        .iter()
        .map(|p| p.comment_rate.max(0.0))
        .fold(0.0_f64, f64::max);

    rsx! {
        div {
            class: "rounded-2xl border border-border bg-background shadow-sm p-4 flex flex-col gap-3 {height_class}",
            // Header
            div { class: "flex items-center justify-between gap-3",
                div { class: "flex flex-col",
                    h2 {
                        class: "text-sm font-semibold text-zinc-900 dark:text-zinc-50",
                        "{title}"
                    }
                    p {
                        class: "text-[10px] text-zinc-500 dark:text-zinc-500",
                        "Ranking posts by comments relative to views."
                    }
                }
                // Placeholder for future filter controls
                div {
                    class: "inline-flex items-center gap-1 px-2 py-1 rounded-full border border-border bg-background",
                    span {
                        class: "text-[9px] font-medium text-zinc-600 dark:text-zinc-400",
                        "comment_rate = comments / views"
                    }
                }
            }

            // Content
            if props.points.is_empty() {
                div {
                    class: "flex-1 flex items-center justify-center",
                    span {
                        class: "text-[11px] text-zinc-500 dark:text-zinc-500",
                        "No comment activity data available for the selected period."
                    }
                }
            } else {
                div {
                    class: "mt-1 flex-1 flex flex-col gap-1.5 overflow-y-auto pr-1",
                    {props.points.iter().enumerate().map(|(idx, p)| {
                        // Compute proportional width; avoid division by zero.
                        let ratio = if max_value > 0.0 {
                            (p.comment_rate.max(0.0) / max_value).clamp(0.05, 1.0)
                        } else {
                            0.5
                        };

                        // Shorten long titles for compact view
                        let title = if p.title.len() > 40 {
                            format!("{}…", &p.title[..37])
                        } else {
                            p.title.clone()
                        };

                        let width_style = format!("width: {}%;", ratio * 100.0);

                        rsx! {
                            div {
                                key: "{p.post_id}-{idx}",
                                class: "flex flex-col gap-0.5",
                                // Row: title + numeric stats
                                div {
                                    class: "flex items-baseline justify-between gap-2",
                                    span {
                                        class: "text-[10px] font-medium text-zinc-800 dark:text-zinc-100 truncate",
                                        "{title}"
                                    }
                                    span {
                                        class: "shrink-0 text-[9px] tabular-nums text-zinc-500 dark:text-zinc-500",
                                        "{p.comments} comments · {p.views} views · {format_rate(p.comment_rate)}"
                                    }
                                }
                                // Bar track
                                div {
                                    class: "w-full h-4 rounded-md border border-border bg-background",
                                    div {
                                        class: "h-full rounded-md bg-emerald-500 transition-all duration-300 ease-out",
                                        style: "{width_style}",
                                    }
                                }
                            }
                        }
                    })}
                }
            }
        }
    }
}

/// Hook-friendly wrapper that binds to `use_analytics().comment_rate`.
///
/// This component:
/// - Reads from the `comment_rate` `StateFrame`
/// - Shows loading / error / empty states
/// - Forwards the resolved points into `CommentRateChart`
///
/// Use this in screens where you want the store-bound chart instead of manually
/// passing `Vec<CommentRatePoint>`.
#[component]
pub fn CommentRateChartFromStore(
    #[props(optional)] title: Option<String>,
    #[props(optional)] height: Option<String>,
    #[props(optional)] max_items: Option<usize>,
) -> Element {
    let analytics = use_analytics();
    let state_signal = &analytics.comment_rate;
    let frame = state_signal.read().clone();

    let title = title.unwrap_or_else(|| "Top posts by comment engagement".to_string());
    let height = height.or_else(|| Some("h-80".to_string()));
    let max_items = max_items.unwrap_or(10);
    let is_loading = matches!(
        frame.status,
        StateFrameStatus::Init | StateFrameStatus::Loading
    );

    match frame.status {
        StateFrameStatus::Init => {
            rsx! {
                SkeletonCommentRateCard { title, height }
            }
        }
        StateFrameStatus::Loading => {
            rsx! {
                SkeletonCommentRateCard { title, height }
            }
        }
        StateFrameStatus::Failed => {
            rsx! {
                div {
                    class: "rounded-2xl border border-border bg-background shadow-none p-4 flex flex-col gap-2 h-48 relative",
                    h2 {
                        class: "text-sm font-semibold text-foreground",
                        "{title}"
                    }
                    ErrorDetails {
                        error: frame.error.clone(),
                        variant: ErrorDetailsVariant::Collapsed,
                        title: Some("Failed to load comment rate analytics".to_string()),
                    }

                    LoadingOverlay { visible: is_loading }
                }
            }
        }
        StateFrameStatus::Success => {
            if let Some(data) = &frame.data {
                let mut points = data.data.clone();
                // Sort descending by comment_rate, then comments
                points.sort_by(|a, b| {
                    b.comment_rate
                        .partial_cmp(&a.comment_rate)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| b.comments.cmp(&a.comments))
                });
                points.truncate(max_items);

                rsx! {
                    CommentRateChart {
                        points,
                        title: Some(title),
                        height
                    }
                }
            } else {
                rsx! {
                    SkeletonCommentRateCard { title, height }
                }
            }
        }
    }
}

/// Skeleton/loading variant matching the chart card shell.
#[component]
fn SkeletonCommentRateCard(title: String, #[props(optional)] height: Option<String>) -> Element {
    let height_class = height.unwrap_or_else(|| "h-80".to_string());

    rsx! {
        div {
            class: "rounded-2xl border border-border bg-background shadow-none p-4 flex flex-col gap-3 {height_class}",
            div { class: "flex items-center justify-between gap-3",
                div { class: "flex flex-col gap-1",
                    h2 {
                        class: "text-sm font-semibold text-zinc-900 dark:text-zinc-50",
                        "{title}"
                    }
                    div {
                        class: "w-40 h-2 rounded-full bg-muted animate-pulse",
                    }
                }
                div {
                    class: "w-24 h-5 rounded-full bg-muted animate-pulse",
                }
            }
            div {
                class: "mt-2 flex-1 flex flex-col gap-2",
                {(0..6).map(|i| {
                    rsx! {
                        div {
                            key: "{i}",
                            class: "flex flex-col gap-1",
                            div {
                                class: "w-32 h-2 rounded-full bg-muted animate-pulse",
                            }
                            div {
                                class: "w-full h-4 rounded-md bg-muted animate-pulse",
                            }
                        }
                    }
                })}
            }
        }
    }
}

/// Format comment_rate as a compact percentage string.
fn format_rate(rate: f64) -> String {
    if !rate.is_finite() || rate <= 0.0 {
        return "0.0%".to_string();
    }

    // rate is comments / views; show as percentage with single decimal.
    let pct = rate * 100.0;
    if pct >= 100.0 {
        format!("{pct:.0}%")
    } else if pct >= 10.0 {
        format!("{pct:.1}%")
    } else {
        format!("{pct:.2}%")
    }
}
