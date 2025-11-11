use dioxus::prelude::*;

use super::interval_selector::IntervalSelector;
use crate::store::{
    AnalyticsEnvelopeResponse, AnalyticsInterval, PageViewPoint, StateFrame, StateFrameStatus,
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
    pub frame:
        StateFrame<AnalyticsEnvelopeResponse<Vec<PageViewPoint>>, crate::store::PageViewsRequest>,
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
            class: "rounded-2xl border border-border bg-background shadow-none \
                    flex flex-col gap-2 {padding}",
            // Header
            div { class: "flex items-center justify-between gap-2",
                div { class: "flex flex-col",
                    h3 {
                        class: "text-sm font-semibold text-zinc-900 dark:text-zinc-100",
                        "{props.title}"
                    }
                    span {
                        class: "text-[10px] uppercase tracking-wide text-zinc-400",
                        "Views vs unique visitors"
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

            // Chart-specific filters panel
            {
                let has_filters = props.on_post_id_change.is_some()
                    || props.on_author_id_change.is_some()
                    || props.on_only_unique_change.is_some();

                if has_filters {
                    rsx! {
                        div {
                            class: "flex flex-wrap items-center gap-3 pt-2 pb-1 border-t border-zinc-200/50 dark:border-zinc-800/50",

                            // Post ID filter
                            if let Some(post_handler) = props.on_post_id_change {
                                div {
                                    class: "flex items-center gap-1.5",
                                    label {
                                        class: "text-[9px] font-medium text-zinc-500 whitespace-nowrap",
                                        "Post ID:"
                                    }
                                    input {
                                        r#type: "number",
                                        value: props.current_post_id.map(|id| id.to_string()).unwrap_or_default(),
                                        placeholder: "All",
                                        oninput: move |evt: Event<FormData>| {
                                            let value = evt.value();
                                            let parsed = if value.is_empty() {
                                                None
                                            } else {
                                                value.parse::<i32>().ok()
                                            };
                                            post_handler.call(parsed);
                                        },
                                        class: "w-20 px-2 py-1 text-[10px] rounded border border-zinc-200 \
                                               dark:border-zinc-800 bg-transparent focus:border-ring \
                                               focus:ring-1 focus:ring-ring/40",
                                    }
                                }
                            }

                            // Author ID filter
                            if let Some(author_handler) = props.on_author_id_change {
                                div {
                                    class: "flex items-center gap-1.5",
                                    label {
                                        class: "text-[9px] font-medium text-zinc-500 whitespace-nowrap",
                                        "Author ID:"
                                    }
                                    input {
                                        r#type: "number",
                                        value: props.current_author_id.map(|id| id.to_string()).unwrap_or_default(),
                                        placeholder: "All",
                                        oninput: move |evt: Event<FormData>| {
                                            let value = evt.value();
                                            let parsed = if value.is_empty() {
                                                None
                                            } else {
                                                value.parse::<i32>().ok()
                                            };
                                            author_handler.call(parsed);
                                        },
                                        class: "w-20 px-2 py-1 text-[10px] rounded border border-zinc-200 \
                                               dark:border-zinc-800 bg-transparent focus:border-ring \
                                               focus:ring-1 focus:ring-ring/40",
                                    }
                                }
                            }

                            // Only unique toggle
                            if let Some(unique_handler) = props.on_only_unique_change {
                                div {
                                    class: "flex items-center gap-1.5",
                                    input {
                                        r#type: "checkbox",
                                        id: "only-unique-toggle",
                                        checked: props.current_only_unique,
                                        onchange: move |evt: Event<FormData>| {
                                            unique_handler.call(evt.checked());
                                        },
                                        class: "w-3 h-3 rounded border-zinc-300 dark:border-zinc-700 \
                                               text-ring focus:ring-2 focus:ring-ring/40",
                                    }
                                    label {
                                        r#for: "only-unique-toggle",
                                        class: "text-[9px] font-medium text-zinc-600 dark:text-zinc-400 \
                                               cursor-pointer select-none",
                                        "Only unique visitors"
                                    }
                                }
                            }
                        }
                    }
                } else {
                    rsx! {}
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
    let padding_top = if compact { "mt-1" } else { "mt-2" };

    rsx! {
        div { class: "flex-1 flex flex-col justify-end gap-2 {padding_top}",
            // "Chart" skeleton
            div {
                class: "w-full {height} rounded-xl bg-muted animate-pulse flex items-end gap-1 px-3 pb-3",
                // Bars skeleton
                for i in 0..18 {
                    {
                        let h = 20 + (i * 3) % 60;
                        rsx! {
                            div {
                                key: "{i}",
                                class: "flex-1 bg-muted rounded-t-md",
                                style: "height: {h}%;"
                            }
                        }
                    }
                }
            }

            // Legend skeleton
            div { class: "flex items-center gap-4 text-[10px] text-zinc-400",
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
    let padding_y = if compact { "py-4" } else { "py-6" };

    rsx! {
        div { class: "flex-1 flex flex-col items-center justify-center gap-2 {padding_y}",
            div { class: "text-[11px] font-medium text-rose-600 dark:text-rose-400",
                "Unable to load page views"
            }
            p { class: "text-[10px] text-zinc-500 text-center max-w-xs",
                "{message}"
            }
            // In a future iteration, we can accept an `on_retry` callback.
        }
    }
}

/// Empty state when the request succeeds but returns no data.
#[component]
fn EmptyState(compact: bool) -> Element {
    let padding_y = if compact { "py-4" } else { "py-6" };

    rsx! {
        div { class: "flex-1 flex flex-col items-center justify-center gap-1 {padding_y}",
            div { class: "text-[11px] font-medium text-zinc-700 dark:text-zinc-300",
                "No page views to display yet"
            }
            p { class: "text-[10px] text-zinc-500 text-center max-w-xs",
                "Traffic data will appear here once your site starts receiving visits."
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

            // Legend
            div { class: "flex items-center justify-between gap-2 text-[9px]",
                div { class: "flex items-center gap-3",
                    LegendPill { color: "bg-sky-400", label: "Views" }
                    LegendPill { color: "bg-emerald-500", label: "Unique visitors" }
                }
                // Max label
                div { class: "text-[8px] text-zinc-400",
                    "Peak: {max_value}"
                }
            }
        }
    }
}

#[component]
fn LegendPill(color: &'static str, label: &'static str) -> Element {
    rsx! {
        div { class: "inline-flex items-center gap-1 text-zinc-500 dark:text-zinc-400",
            span { class: "w-2 h-2 rounded-full {color}" }
            span { "{label}" }
        }
    }
}
