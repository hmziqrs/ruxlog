use dioxus::prelude::*;

use super::interval_selector::IntervalSelector;
use oxstore::{StateFrame, StateFrameStatus};
use ruxlog_shared::store::analytics::{
    AnalyticsEnvelopeResponse, AnalyticsInterval, PageViewPoint,
};

/// Simple typed props for the page views chart.
///
/// Exposes:
/// - `frame`: current state for the underlying analytics request
/// - `title`: optional title for the card
/// - `height`: optional height (Tailwind class, default "h-72")
/// - `compact`: optional flag to tweak padding/typography for dense layouts
/// - Filter callbacks for interval, post_id, author_id, only_unique
#[derive(Props, PartialEq, Clone)]
pub struct PageViewsChartProps {
    /// State frame wrapping `AnalyticsEnvelopeResponse<Vec<PageViewPoint>>`.
    pub frame: StateFrame<
        AnalyticsEnvelopeResponse<Vec<PageViewPoint>>,
        ruxlog_shared::store::PageViewsRequest,
    >,
    /// Optional chart title.
    #[props(default = "Page views".to_string())]
    pub title: String,
    /// Tailwind height class for the chart container.
    #[props(default = "h-72".to_string())]
    pub height: String,
    /// Render with slightly more compact paddings.
    #[props(default = false)]
    pub compact: bool,

    // Filter-related props
    /// Current interval grouping
    #[props(default = AnalyticsInterval::Day)]
    pub current_interval: AnalyticsInterval,
    /// Callback when interval changes
    #[props(default)]
    pub on_interval_change: Option<EventHandler<AnalyticsInterval>>,
    /// Current post ID filter (None = all posts)
    #[props(default)]
    pub current_post_id: Option<i32>,
    /// Callback when post ID filter changes
    #[props(default)]
    pub on_post_id_change: Option<EventHandler<Option<i32>>>,
    /// Current author ID filter (None = all authors)
    #[props(default)]
    pub current_author_id: Option<i32>,
    /// Callback when author ID filter changes
    #[props(default)]
    pub on_author_id_change: Option<EventHandler<Option<i32>>>,
    /// Current "only unique" toggle state
    #[props(default = false)]
    pub current_only_unique: bool,
    /// Callback when "only unique" toggle changes
    #[props(default)]
    pub on_only_unique_change: Option<EventHandler<bool>>,
}

/// High-level page views chart wrapper.
///
/// Responsibilities:
/// - Render a consistent card shell (border, bg, padding).
/// - Interpret `StateFrame` (loading, error, empty, ready).
/// - When ready, pass the data into the chart body.
/// - For now, includes a minimal SVG-based placeholder chart until `dioxus-charts`
///   is wired into the workspace (see analytics-dashboard-charts-plan.md).
#[component]
pub fn PageViewsChart(props: PageViewsChartProps) -> Element {
    let status = props.frame.status;
    let body = if status == StateFrameStatus::Init || status == StateFrameStatus::Loading {
        rsx! {
            LoadingState {
                height: props.height.clone(),
                compact: props.compact,
            }
        }
    } else if status == StateFrameStatus::Failed {
        rsx! {
            ErrorState {
                message: props
                    .frame
                    .error_message()
                    .unwrap_or_else(|| "Unable to load page views data.".to_string()),
                compact: props.compact,
            }
        }
    } else if status == StateFrameStatus::Success {
        let response_opt = props.frame.data;
        match response_opt {
            None => rsx! {
                EmptyState { compact: props.compact }
            },
            Some(envelope) => {
                let points = &envelope.data;
                if points.is_empty() {
                    rsx! {
                        EmptyState { compact: props.compact }
                    }
                } else {
                    rsx! {
                        ChartBody {
                            points: points.to_vec(),
                            height: props.height.clone(),
                            compact: props.compact,
                        }
                    }
                }
            }
        }
    } else {
        rsx! {
            EmptyState { compact: props.compact }
        }
    };

    let padding = if props.compact { "p-4" } else { "p-5" };

    rsx! {
        div {
            class: "rounded-xl border border-border bg-card \
                    flex flex-col {padding}",
            // Header
            div { class: "flex items-center justify-between gap-2 mb-3",
                div { class: "flex flex-col gap-0.5",
                    h3 {
                        class: "text-sm font-semibold text-foreground",
                        "{props.title}"
                    }
                    span {
                        class: "text-xs text-muted-foreground",
                        "Total views vs unique visitors over time"
                    }
                }

                // Interval selector + unique toggle in a row
                div {
                    class: "flex items-center gap-3",
                    // Only unique toggle (simplified)
                    if let Some(unique_handler) = props.on_only_unique_change {
                        label {
                            class: "flex items-center gap-1.5 cursor-pointer",
                            input {
                                r#type: "checkbox",
                                checked: props.current_only_unique,
                                onchange: move |evt: Event<FormData>| {
                                    unique_handler.call(evt.checked());
                                },
                                class: "w-3.5 h-3.5 rounded border-border \
                                       text-primary focus:ring-2 focus:ring-ring/40",
                            }
                            span {
                                class: "text-xs text-muted-foreground",
                                "Unique only"
                            }
                        }
                    }
                    // Interval selector
                    if let Some(handler) = props.on_interval_change {
                        IntervalSelector {
                            current: props.current_interval,
                            on_change: handler,
                            label: "".to_string(),
                        }
                    }
                }
            }

            // Body (loading/error/chart)
            {body}
        }
    }
}

/// Skeleton/loading state while analytics request is in-flight.
#[component]
fn LoadingState(height: String, compact: bool) -> Element {
    let _padding_top = if compact { "mt-1" } else { "mt-2" };

    rsx! {
        div { class: "flex-1 flex flex-col justify-end gap-3",
            // "Chart" skeleton
            div {
                class: "w-full {height} rounded-lg bg-muted/50 flex items-end gap-1 px-3 pb-3",
                // Bars skeleton
                for i in 0..12 {
                    {
                        let h = 25 + (i * 5) % 55;
                        rsx! {
                            div {
                                key: "{i}",
                                class: "flex-1 bg-muted rounded-t animate-pulse",
                                style: "height: {h}%;"
                            }
                        }
                    }
                }
            }

            // Legend skeleton
            div { class: "flex items-center gap-4",
                LegendPillSkeleton { label: "Views" }
                LegendPillSkeleton { label: "Unique" }
            }
        }
    }
}

#[component]
fn LegendPillSkeleton(label: &'static str) -> Element {
    rsx! {
        div { class: "flex items-center gap-1",
            span { class: "inline-block w-2 h-2 rounded-full bg-muted" }
            span { class: "h-2 w-10 rounded-full bg-muted" }
            span { class: "sr-only", "{label}" }
        }
    }
}

/// Error state aligned with the shared analytics toast/error patterns.
#[component]
fn ErrorState(message: String, compact: bool) -> Element {
    let padding_y = if compact { "py-6" } else { "py-10" };

    rsx! {
        div { class: "flex-1 flex flex-col items-center justify-center gap-2 {padding_y}",
            // Error icon
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
            div { class: "text-sm font-medium text-foreground",
                "Unable to load page views"
            }
            p { class: "text-xs text-muted-foreground text-center max-w-xs",
                "{message}"
            }
        }
    }
}

/// Empty state when the request succeeds but returns no data.
#[component]
fn EmptyState(compact: bool) -> Element {
    let padding_y = if compact { "py-6" } else { "py-10" };

    rsx! {
        div { class: "flex-1 flex flex-col items-center justify-center gap-2 {padding_y}",
            // Empty icon
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
                        d: "M3 13.125C3 12.504 3.504 12 4.125 12h2.25c.621 0 1.125.504 1.125 1.125v6.75C7.5 20.496 6.996 21 6.375 21h-2.25A1.125 1.125 0 0 1 3 19.875v-6.75ZM9.75 8.625c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125v11.25c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 0 1-1.125-1.125V8.625ZM16.5 4.125c0-.621.504-1.125 1.125-1.125h2.25C20.496 3 21 3.504 21 4.125v15.75c0 .621-.504 1.125-1.125 1.125h-2.25a1.125 1.125 0 0 1-1.125-1.125V4.125Z"
                    }
                }
            }
            div { class: "text-sm font-medium text-foreground",
                "No page views yet"
            }
            p { class: "text-xs text-muted-foreground text-center max-w-xs",
                "Traffic data will appear here once your blog starts receiving visits."
            }
        }
    }
}

/// Minimal chart body.
///
/// For now, this uses pure SVG to visualize:
/// - Blue line/area for `views`
/// - Emerald line for `unique_visitors`
///
/// Once `dioxus-charts` is added to `Cargo.toml`, this function can be
/// refactored to use its primitives without changing the public API.
#[component]
fn ChartBody(points: Vec<PageViewPoint>, height: String, compact: bool) -> Element {
    if points.is_empty() {
        return rsx! { EmptyState { compact: compact } };
    }

    // Compute min/max for scaling
    let mut max_value: i64 = 0;
    for p in &points {
        if p.views > max_value {
            max_value = p.views;
        }
        if p.unique_visitors > max_value {
            max_value = p.unique_visitors;
        }
    }
    if max_value == 0 {
        return rsx! { EmptyState { compact: compact } };
    }

    let count = points.len() as f32;
    let width = 100.0_f32;
    let height_vb = 40.0_f32; // virtual SVG height
    let pad_x = 3.0_f32;
    let pad_y = 4.0_f32;
    let usable_width = width - pad_x * 2.0;
    let usable_height = height_vb - pad_y * 2.0;

    let scale_x = if count <= 1.0 {
        0.0
    } else {
        usable_width / (count - 1.0)
    };
    let scale_y = |value: i64| -> f32 {
        let v = value as f32 / max_value as f32;
        pad_y + (1.0 - v) * usable_height
    };

    // Build polyline points for views and uniques
    let mut views_points = String::new();
    let mut unique_points = String::new();

    for (i, p) in points.iter().enumerate() {
        let x = pad_x + i as f32 * scale_x;
        let y_views = scale_y(p.views);
        let y_uniques = scale_y(p.unique_visitors);

        if i > 0 {
            views_points.push(' ');
            unique_points.push(' ');
        }

        views_points.push_str(&format!("{:.3},{:.3}", x, y_views));
        unique_points.push_str(&format!("{:.3},{:.3}", x, y_uniques));
    }

    let padding_top = if compact { "mt-1" } else { "mt-2" };

    rsx! {
        div { class: "flex-1 flex flex-col gap-2 {padding_top}",
            // SVG chart
            div {
                class: "relative w-full {height}",
                svg {
                    class: "w-full h-full",
                    view_box: "0 0 {width} {height_vb}",
                    xmlns: "http://www.w3.org/2000/svg",

                    // Grid background lines (simple)
                    {
                        (0..=4).map(|i| {
                            let y = pad_y + (usable_height * i as f32 / 4.0);
                            rsx! {
                                line {
                                    key: "grid-{i}",
                                    x1: "{pad_x}",
                                    y1: "{y}",
                                    x2: "{width - pad_x}",
                                    y2: "{y}",
                                    stroke: "currentColor",
                                    class: "text-zinc-200/70 dark:text-zinc-900/80",
                                    "stroke-width": "0.2",
                                }
                            }
                        })
                    }

                    // Area under "views" line (subtle)
                    {
                        if points.len() >= 2 {
                            let mut area = String::new();
                            // Start at bottom-left
                            if let Some(first) = points.first() {
                                let x0 = pad_x;
                                let y0 = scale_y(first.views);
                                area.push_str(&format!("{:.3},{:.3} ", x0, y0));
                                for (i, p) in points.iter().enumerate() {
                                    let x = pad_x + i as f32 * scale_x;
                                    let y = scale_y(p.views);
                                    area.push_str(&format!("{:.3},{:.3} ", x, y));
                                }
                                // Close down to baseline
                                if let Some(_last) = points.last() {
                                    let x_last = pad_x + (points.len() - 1) as f32 * scale_x;
                                    let baseline = scale_y(0);
                                    area.push_str(&format!("{:.3},{:.3} ", x_last, baseline));
                                    area.push_str(&format!("{:.3},{:.3}", x0, baseline));
                                }

                                rsx! {
                                    polygon {
                                        points: "{area}",
                                        fill: "url(#viewsGradient)",
                                        "fill-opacity": "0.18",
                                    }
                                }
                            } else {
                                rsx! {}
                            }
                        } else {
                            rsx! {}
                        }
                    }

                    defs {
                        linearGradient {
                            id: "viewsGradient",
                            x1: "0", y1: "0", x2: "0", y2: "1",
                            stop { offset: "0%", "stop-color": "#38bdf8" }
                            stop { offset: "100%", "stop-color": "#38bdf8", "stop-opacity": "0" }
                        }
                    }

                    // Views line (primary)
                    polyline {
                        points: "{views_points}",
                        fill: "none",
                        stroke: "#38bdf8", // sky-400
                        "stroke-width": "0.9",
                        "stroke-linecap": "round",
                        "stroke-linejoin": "round",
                    }

                    // Unique visitors line (secondary)
                    polyline {
                        points: "{unique_points}",
                        fill: "none",
                        stroke: "#22c55e", // emerald-500
                        "stroke-width": "0.7",
                        "stroke-linecap": "round",
                        "stroke-linejoin": "round",
                        "stroke-dasharray": "2 2",
                    }
                }
            }

            // Legend and stats
            div { class: "flex items-center justify-between gap-2 pt-2 border-t border-border",
                div { class: "flex items-center gap-4",
                    LegendPill { color: "bg-sky-400", label: "Views" }
                    LegendPill { color: "bg-emerald-500", label: "Unique visitors", dashed: true }
                }
                // Peak label
                div { class: "text-xs text-muted-foreground",
                    "Peak: "
                    span { class: "font-medium text-foreground", "{max_value}" }
                }
            }
        }
    }
}

#[component]
fn LegendPill(color: &'static str, label: &'static str, #[props(default = false)] dashed: bool) -> Element {
    rsx! {
        div { class: "inline-flex items-center gap-1.5 text-xs text-muted-foreground",
            if dashed {
                span { class: "w-3 h-0.5 rounded {color}" }
            } else {
                span { class: "w-2.5 h-2.5 rounded-sm {color}" }
            }
            span { "{label}" }
        }
    }
}
