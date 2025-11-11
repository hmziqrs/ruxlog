use dioxus::prelude::*;

use crate::store::{AnalyticsEnvelopeResponse, RegistrationTrendPoint, StateFrame};

/// Props for `RegistrationTrendChart`.
///
/// Keep this minimal and typed:
/// - `frame` is the state wrapper produced by `use_analytics().registration_trends`
/// - `title` and `height` customize the card shell.
/// - `on_retry` lets the parent trigger a refetch when errors occur.
#[derive(Props, PartialEq, Clone)]
pub struct RegistrationTrendChartProps {
    /// State frame containing analytics envelope response for registration trends.
    pub frame: StateFrame<
        AnalyticsEnvelopeResponse<Vec<RegistrationTrendPoint>>,
        crate::store::RegistrationTrendsRequest,
    >,
    /// Optional title for the chart card.
    #[props(default = "New user registrations".to_string())]
    pub title: String,
    /// Tailwind height class (e.g. "h-64", "h-80").
    #[props(default = "h-64".to_string())]
    pub height: String,
    /// Retry callback invoked from error state UI.
    #[props(default = None)]
    pub on_retry: Option<EventHandler<()>>,
}

/// Registration trend chart card.
///
/// This is a scoped, presentation-only component:
/// - Responsible for rendering loading/empty/error/data states for the registration trends.
/// - The actual fetching logic should live in the screen using `use_analytics().fetch_registration_trends(...)`.
/// - The real chart visualization should be implemented with `dioxus-charts` in a follow-up step.
///   For now this scaffolds a consistent layout and basic SVG-based line visualization so the screen
///   can already integrate this component.
#[component]
pub fn RegistrationTrendChart(props: RegistrationTrendChartProps) -> Element {
    let RegistrationTrendChartProps {
        frame,
        title,
        height,
        on_retry,
    } = props;

    // Basic state helpers. These mirror the typical `StateFrame` API pattern:
    // adjust if the actual implementation differs.
    let is_loading = frame.is_loading();
    let has_error = frame.error.is_some();
    let data_opt = frame.data.as_ref().map(|env| &env.data);

    let content = if is_loading {
        rsx! {
            div { class: "flex flex-col gap-2 animate-pulse",
                div { class: "w-32 h-4 bg-zinc-200/70 dark:bg-zinc-800/70 rounded" }
                div { class: "w-full h-8 bg-zinc-200/70 dark:bg-zinc-800/70 rounded" }
                div { class: "w-full flex-1 bg-zinc-200/70 dark:bg-zinc-800/70 rounded" }
            }
        }
    } else if has_error {
        rsx! {
            div { class: "flex flex-col items-start gap-2 text-sm",
                div {
                    class: "text-rose-600 dark:text-rose-400 font-medium",
                    "Unable to load registration trends"
                }
                div {
                    class: "text-xs text-zinc-500 dark:text-zinc-400",
                    "Check your connection or try refreshing the data."
                }
                if let Some(on_retry) = on_retry {
                    button {
                        class: "inline-flex items-center px-3 py-1.5 rounded-md text-xs font-medium
                                bg-zinc-900 text-zinc-50 hover:bg-zinc-800
                                dark:bg-zinc-100 dark:text-zinc-900 dark:hover:bg-zinc-200
                                transition-colors",
                        onclick: move |_| on_retry.call(()),
                        "Retry"
                    }
                }
            }
        }
    } else if let Some(points) = data_opt {
        if points.is_empty() {
            rsx! {
                div { class: "flex flex-col items-start justify-center h-full gap-1 text-xs",
                    span { class: "text-zinc-500 dark:text-zinc-400",
                        "No registration activity in the selected period."
                    }
                    span { class: "text-zinc-400 dark:text-zinc-500",
                        "Adjust filters to see trends over a different time range."
                    }
                }
            }
        } else {
            render_trend_chart(points)
        }
    } else {
        // No data, not loading, no explicit error: treat as soft-empty.
        rsx! {
            div { class: "flex flex-col items-start justify-center h-full gap-1 text-xs",
                span { class: "text-zinc-500 dark:text-zinc-400",
                    "Registration trends are not available yet."
                }
                span { class: "text-zinc-400 dark:text-zinc-500",
                    "Use the analytics controls to fetch data for this period."
                }
            }
        }
    };

    rsx! {
        div {
            class: "rounded-2xl border border-zinc-200/80 dark:border-zinc-800/80 \
                    bg-zinc-50/70 dark:bg-zinc-950/60 shadow-none backdrop-blur-sm \
                    p-4 flex flex-col gap-2 {height}",
            // Header
            div { class: "flex items-center justify-between gap-2",
                div { class: "flex flex-col gap-0.5",
                    h3 {
                        class: "text-sm font-semibold text-zinc-900 dark:text-zinc-100",
                        "{title}"
                    }
                    span {
                        class: "text-[10px] uppercase tracking-wide text-zinc-400",
                        "New accounts over time"
                    }
                }
                // Placeholder legend (colors chosen to be consistent with other charts).
                div { class: "flex items-center gap-2 text-[9px] text-zinc-500 dark:text-zinc-400",
                    div { class: "w-2 h-2 rounded-full bg-sky-500/90" }
                    span { "New users" }
                }
            }

            // Chart body
            div { class: "mt-1 flex-1 min-h-0",
                {content}
            }
        }
    }
}

/// Minimal inline SVG line chart for registration trends.
///
/// This is intentionally simple and self-contained so the component is
/// usable immediately. In a later step, this can be replaced with a proper
/// `dioxus-charts` implementation without changing the public props.
fn render_trend_chart(points: &[RegistrationTrendPoint]) -> Element {
    // Extract numeric values
    let mut max_y = 0_i64;
    for p in points {
        if p.new_users > max_y {
            max_y = p.new_users;
        }
    }
    // Avoid division-by-zero; also handles negative/empty gracefully
    let max_y = if max_y <= 0 { 1 } else { max_y };

    let count = points.len().max(2) as f32;
    let width = 100_f32;
    let height = 40_f32;

    // Generate polyline points in viewBox coordinates
    let mut path_points = String::new();
    for (idx, p) in points.iter().enumerate() {
        let x = (idx as f32 / (count - 1.0)) * width;
        let y_ratio = p.new_users as f32 / max_y as f32;
        let y = height - (y_ratio * (height - 4.0)) - 2.0; // padding
        if !path_points.is_empty() {
            path_points.push(' ');
        }
        path_points.push_str(&format!("{:.2},{:.2}", x, y));
    }

    // Simple y-axis labels: 0 and max
    let y_max_label = max_y;

    rsx! {
        div { class: "w-full h-full flex flex-col gap-1",
            // Inline SVG chart
            svg {
                class: "w-full h-full text-sky-500",
                view_box: "0 0 100 40",
                preserve_aspect_ratio: "none",

                // Grid lines (subtle)
                line {
                    x1: "0", y1: "38",
                    x2: "100", y2: "38",
                    class: "stroke-zinc-200 dark:stroke-zinc-800",
                    "stroke-width": "0.3"
                }
                line {
                    x1: "0", y1: "2",
                    x2: "100", y2: "2",
                    class: "stroke-zinc-200/70 dark:stroke-zinc-800/70",
                    "stroke-width": "0.2"
                }

                // Area under line
                if !path_points.is_empty() {
                    polygon {
                        class: "fill-sky-500/8",
                        points: format!("0,40 {points} 100,40", points = path_points),
                    }
                }

                // Trend line
                if !path_points.is_empty() {
                    polyline {
                        class: "fill-none stroke-sky-500",
                        "stroke-width": "0.8",
                        "stroke-linejoin": "round",
                        "stroke-linecap": "round",
                        points: "{path_points}",
                    }
                }

                // Dots
                for (idx, p) in points.iter().enumerate() {
                    {
                        let x = (idx as f32 / (count - 1.0)) * width;
                        let y_ratio = p.new_users as f32 / max_y as f32;
                        let y = height - (y_ratio * (height - 4.0)) - 2.0;
                        rsx! { circle {
                            cx: format!("{:.2}", x),
                            cy: format!("{:.2}", y),
                            r: "1.1",
                            class: "fill-sky-500",
                        }}
                    }
                }
            }

            // Simple axis labels row (buckets)
            div { class: "flex justify-between items-baseline gap-1 text-[8px] text-zinc-400",
                span { "0" }
                span { "{y_max_label}" }
            }

            // X labels (first and last bucket for now to avoid overcrowding)
            if let Some(first) = points.first() {
                if let Some(last) = points.last() {
                    div { class: "flex justify-between items-baseline gap-1 text-[8px] text-zinc-400",
                        span { "{first.bucket}" }
                        span { "{last.bucket}" }
                    }
                }
            }
        }
    }
}
