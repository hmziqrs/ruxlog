use dioxus::prelude::*;

use crate::store::{AnalyticsEnvelopeResponse, MediaUploadPoint, StateFrame, StateFrameStatus};

/// Props for `MediaUploadTrendsChart`.
///
/// This component is intentionally thin:
/// - It consumes a `StateFrame` for media upload trends data.
/// - It renders a styled card shell with loading/error/empty states handled.
/// - When `dioxus-charts` is available, hook it up in the marked section below.
#[derive(Props, PartialEq, Clone)]
pub struct MediaUploadTrendsChartProps {
    /// Frame containing media upload trend points.
    pub frame: StateFrame<
        AnalyticsEnvelopeResponse<Vec<MediaUploadPoint>>,
        crate::store::MediaUploadRequest,
    >,
    /// Optional title for the chart card.
    #[props(default = "Media Upload Trends".to_string())]
    pub title: String,
    /// Tailwind height classes for the chart area.
    #[props(default = "h-64".to_string())]
    pub height_class: String,
}

/// Media upload trends visualization:
/// - Bars: upload_count
/// - Line: avg_size_mb (later: mapped via secondary axis when supported)
///
/// NOTE: This is scaffolded without `dioxus-charts` wiring. Once the dependency
/// is added, replace the placeholder graph area with real chart primitives,
/// using this component as a stable shell.
#[component]
pub fn MediaUploadTrendsChart(props: MediaUploadTrendsChartProps) -> Element {
    let MediaUploadTrendsChartProps {
        frame,
        title,
        height_class,
    } = props;

    let status = frame.status;
    let error = frame.error;
    let data = frame
        .data
        .as_ref()
        .map(|env| env.data.clone())
        .unwrap_or_default();

    let is_loading = matches!(status, StateFrameStatus::Init | StateFrameStatus::Loading);
    let has_error = error.is_some();
    let is_empty = !is_loading && !has_error && data.is_empty();

    // Derive basic stats for quick context badges.
    let (total_uploads, last_avg_size_mb) = if data.is_empty() {
        (0_i64, None)
    } else {
        let total: i64 = data.iter().map(|p| p.upload_count).sum();
        let last_avg = data.last().map(|p| p.avg_size_mb);
        (total, last_avg)
    };

    rsx! {
        div {
            class: "rounded-2xl border border-border bg-background shadow-none \
                    flex flex-col gap-3 p-4",

            // Header
            div {
                class: "flex items-center justify-between gap-3",
                div {
                    class: "flex flex-col",
                    h3 {
                        class: "text-sm font-semibold text-zinc-900 dark:text-zinc-50",
                        "{title}"
                    }
                    p {
                        class: "text-[11px] text-zinc-500",
                        "Track how many media files are uploaded over time and how their average size evolves."
                    }
                }

                // Quick stats
                div {
                    class: "flex flex-col items-end gap-0.5 text-[10px]",
                    span {
                        class: "px-2 py-0.5 rounded-full bg-background border border-border text-foreground",
                        "Total uploads: {total_uploads}"
                    }
                    if let Some(avg) = last_avg_size_mb {
                        span {
                            class: "px-2 py-0.5 rounded-full bg-background border border-border text-foreground",
                            "Last bucket avg size: {avg:.1} MB"
                        }
                    }
                }
            }

            // Content area
            if is_loading {
                // Loading skeleton
                div {
                    class: "animate-pulse flex-1 flex flex-col gap-3 mt-1",
                    div {
                        class: "h-4 w-32 bg-muted rounded-full"
                    }
                    div {
                        class: "h-3 w-24 bg-muted rounded-full"
                    }
                    div {
                        class: "{height_class} mt-1 rounded-xl bg-muted"
                    }
                }
            } else if has_error {
                // Error state
                {
                    let err = error
                        .as_ref()
                        .map(|e| e.message())
                        .unwrap_or_else(|| "Unable to load media upload trends.".to_string());

                    rsx! { div {
                    class: "flex flex-col gap-2 mt-1",
                    div {
                        class: "text-[11px] px-2 py-1.5 rounded-lg bg-background border border-destructive/40 text-destructive",
                        span { class: "font-semibold mr-1", "Error:" }
                        span { "{err}" }
                    }
                    div {
                        class: "{height_class} rounded-xl border border-dashed border-destructive/40 flex items-center justify-center bg-background",
                        span {
                            class: "text-[10px] text-destructive",
                            "Chart unavailable due to an error."
                        }
                    }
                }}
                }
            } else if is_empty {
                // Empty state
                div {
                    class: "flex flex-col gap-2 mt-1",
                    div {
                        class: "text-[11px] px-2 py-1.5 rounded-lg bg-background border border-dashed border-border text-muted-foreground",
                        "No media upload data available for the selected period."
                    }
                    div {
                        class: "{height_class} rounded-xl flex items-center justify-center bg-background",
                        span {
                            class: "text-[10px] text-muted-foreground",
                            "Uploads activity will appear here once there is data."
                        }
                    }
                }
            } else {
                // Data state: placeholder chart scaffold.
                //
                // TODO (charts): Integrate `dioxus-charts` here. Suggested mapping:
                // - X axis: point.bucket (chronological)
                // - Primary Y axis: upload_count (bars)
                // - Secondary Y axis: avg_size_mb (line)
                //
                // For now, render a minimal preview-like visualization so layout is stable.
                div {
                    class: "flex flex-col gap-2 mt-1",
                    // Legend
                    div {
                        class: "flex items-center gap-3 text-[9px] text-muted-foreground",
                        div {
                            class: "flex items-center gap-1",
                            span { class: "w-3 h-1.5 rounded-full bg-primary" }
                            span { "Uploads" }
                        }
                        div {
                            class: "flex items-center gap-1",
                            span { class: "w-3 h-[2px] rounded-full bg-primary" }
                            span { "Avg size (MB)" }
                        }
                    }

                    // Simple SVG-ish bars/line style scaffold using divs
                    div {
                        class: format!(
                            "relative {height_class} mt-1 rounded-xl border border-border \
                             bg-background overflow-hidden px-3 pt-3 pb-4 flex items-end gap-1.5"
                        ),

                        // Compute max for normalization (avoid division by zero).
                        {
                            let _max_uploads = data
                                .iter()
                                .map(|p| p.upload_count.max(0))
                                .max()
                                .unwrap_or(1) as f64;
                            let _max_avg = data
                                .iter()
                                .map(|p| p.avg_size_mb.max(0.0))
                                .fold(0.0_f64, f64::max)
                                .max(1.0);

                            // TODO: restore full chart rendering with proper chart primitives
                            rsx! { div { class: "text-center text-[10px] text-zinc-500", "{data.len()} uploads" } }
                        }
                    }

                    // Footer hint
                    div {
                        class: "flex justify-between items-center mt-1 text-[8px] text-zinc-400",
                        span { "Visual scale is relative to current dataset." }
                        span { "Hook up full dioxus-charts here for production." }
                    }
                }
            }
        }
    }
}
