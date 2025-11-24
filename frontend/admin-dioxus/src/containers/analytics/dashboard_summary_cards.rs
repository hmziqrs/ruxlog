use dioxus::prelude::*;
use oxstore::{StateFrame, StateFrameStatus};

use ruxlog_shared::store::analytics::{
    AnalyticsEnvelopeResponse, DashboardSummaryData, DashboardSummaryRequest,
};

/// Props:
/// - `frame`: state frame for the dashboard summary analytics request
/// - `title`: optional header title
/// - `description`: optional header description
#[derive(Props, PartialEq, Clone)]
pub struct DashboardSummaryCardsProps {
    pub frame: StateFrame<AnalyticsEnvelopeResponse<DashboardSummaryData>, DashboardSummaryRequest>,
    #[props(optional, into)]
    pub title: Option<String>,
    #[props(optional, into)]
    pub description: Option<String>,
}

/// High-level summary cards grid for the dashboard.
/// This is intentionally UI-focused and does not perform its own fetching;
/// the parent should wire it to `use_analytics().dashboard_summary`.
#[component]
pub fn DashboardSummaryCards(props: DashboardSummaryCardsProps) -> Element {
    let title = props
        .title
        .clone()
        .unwrap_or_else(|| "Overview".to_string());
    let description = props
        .description
        .clone()
        .unwrap_or_else(|| "Key metrics for users, content, engagement, and media.".to_string());

    let status = props.frame.status;
    let is_loading = matches!(status, StateFrameStatus::Init | StateFrameStatus::Loading);
    let is_error = matches!(status, StateFrameStatus::Failed);
    let has_data = props.frame.data.is_some();

    // Basic error banner – parent can choose to hide component when errored instead.
    let error_message = if is_error {
        Some(
            props
                .frame
                .error_message()
                .unwrap_or_else(|| "Unable to load dashboard summary.".to_string()),
        )
    } else {
        None
    };

    rsx! {
        section {
            class: "w-full space-y-3",
            // Header (only show if title was explicitly provided)
            if props.title.is_some() {
                div {
                    class: "flex flex-col gap-1",
                    h2 {
                        class: "text-lg font-semibold text-zinc-900 dark:text-zinc-50",
                        "{title}"
                    }
                    p {
                        class: "text-xs text-zinc-500 dark:text-zinc-400",
                        "{description}"
                    }
                }
            }

            // Error state
            if let Some(msg) = error_message {
                div {
                    class: "rounded-xl border border-destructive/40 bg-background text-destructive text-xs px-3 py-2 flex items-start gap-2",
                    span {
                        class: "mt-0.5 h-1.5 w-1.5 rounded-full bg-destructive animate-pulse",
                    }
                    span { "{msg}" }
                }
            }

            // Cards grid: loading skeletons, data, or neutral empty state.
            if is_loading && !has_data {
                SummarySkeletonGrid {}
            } else if let Some(envelope) = &props.frame.data {
                SummaryCardsGrid { summary: envelope.data.clone() }
            } else if !is_error {
                // Empty but not error/loading; show soft hint.
                div {
                    class: "rounded-xl border border-dashed border-border bg-background px-3 py-2 text-[10px] text-muted-foreground",
                    "No summary data yet. Once analytics events flow in, key metrics will appear here."
                }
            }
        }
    }
}

#[component]
fn SummaryCardsGrid(summary: DashboardSummaryData) -> Element {
    rsx! {
        div {
            class: "grid grid-cols-2 lg:grid-cols-4 gap-3",
            // Posts - Primary metric for a blog
            SummaryCard {
                label: "Published posts",
                primary_value: format_number(summary.posts.published),
                secondary_label: "Drafts",
                secondary_value: format_number(summary.posts.drafts),
                icon: rsx! {
                    svg {
                        class: "w-4 h-4",
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
                },
                accent_class: "text-emerald-500",
            }
            // Views - Key engagement metric
            SummaryCard {
                label: "Page views",
                primary_value: format_number(summary.posts.views_in_period),
                secondary_label: "This period",
                secondary_value: "".to_string(),
                icon: rsx! {
                    svg {
                        class: "w-4 h-4",
                        xmlns: "http://www.w3.org/2000/svg",
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke_width: "1.5",
                        stroke: "currentColor",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            d: "M2.036 12.322a1.012 1.012 0 0 1 0-.639C3.423 7.51 7.36 4.5 12 4.5c4.638 0 8.573 3.007 9.963 7.178.07.207.07.431 0 .639C20.577 16.49 16.64 19.5 12 19.5c-4.638 0-8.573-3.007-9.963-7.178Z"
                        }
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            d: "M15 12a3 3 0 1 1-6 0 3 3 0 0 1 6 0Z"
                        }
                    }
                },
                accent_class: "text-sky-500",
            }
            // Comments - Engagement
            SummaryCard {
                label: "Comments",
                primary_value: format_number(summary.engagement.comments_in_period),
                secondary_label: "This period",
                secondary_value: "".to_string(),
                icon: rsx! {
                    svg {
                        class: "w-4 h-4",
                        xmlns: "http://www.w3.org/2000/svg",
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke_width: "1.5",
                        stroke: "currentColor",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            d: "M8.625 12a.375.375 0 1 1-.75 0 .375.375 0 0 1 .75 0Zm0 0H8.25m4.125 0a.375.375 0 1 1-.75 0 .375.375 0 0 1 .75 0Zm0 0H12m4.125 0a.375.375 0 1 1-.75 0 .375.375 0 0 1 .75 0Zm0 0h-.375M21 12c0 4.556-4.03 8.25-9 8.25a9.764 9.764 0 0 1-2.555-.337A5.972 5.972 0 0 1 5.41 20.97a5.969 5.969 0 0 1-.474-.065 4.48 4.48 0 0 0 .978-2.025c.09-.457-.133-.901-.467-1.226C3.93 16.178 3 14.189 3 12c0-4.556 4.03-8.25 9-8.25s9 3.694 9 8.25Z"
                        }
                    }
                },
                accent_class: "text-violet-500",
            }
            // Media - Content assets
            SummaryCard {
                label: "Media files",
                primary_value: format_number(summary.media.total_files),
                secondary_label: "New uploads",
                secondary_value: format_number(summary.media.uploads_in_period),
                icon: rsx! {
                    svg {
                        class: "w-4 h-4",
                        xmlns: "http://www.w3.org/2000/svg",
                        fill: "none",
                        view_box: "0 0 24 24",
                        stroke_width: "1.5",
                        stroke: "currentColor",
                        path {
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            d: "m2.25 15.75 5.159-5.159a2.25 2.25 0 0 1 3.182 0l5.159 5.159m-1.5-1.5 1.409-1.409a2.25 2.25 0 0 1 3.182 0l2.909 2.909m-18 3.75h16.5a1.5 1.5 0 0 0 1.5-1.5V6a1.5 1.5 0 0 0-1.5-1.5H3.75A1.5 1.5 0 0 0 2.25 6v12a1.5 1.5 0 0 0 1.5 1.5Zm10.5-11.25h.008v.008h-.008V8.25Zm.375 0a.375.375 0 1 1-.75 0 .375.375 0 0 1 .75 0Z"
                        }
                    }
                },
                accent_class: "text-amber-500",
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct SummaryCardProps {
    label: &'static str,
    primary_value: String,
    secondary_label: &'static str,
    secondary_value: String,
    icon: Element,
    accent_class: &'static str,
}

#[component]
fn SummaryCard(props: SummaryCardProps) -> Element {
    rsx! {
        div {
            class: "\
                group relative flex flex-col gap-3 \
                rounded-xl border border-border \
                bg-card \
                p-4 \
                transition-all duration-200 \
                hover:border-border/80 hover:shadow-sm",
            // Header with icon
            div {
                class: "flex items-center justify-between",
                span {
                    class: "text-xs font-medium text-muted-foreground",
                    "{props.label}"
                }
                span {
                    class: "text-muted-foreground/60 {props.accent_class}",
                    {props.icon}
                }
            }
            // Primary value
            div {
                class: "flex items-baseline gap-2",
                span {
                    class: "text-2xl font-semibold tracking-tight text-foreground",
                    "{props.primary_value}"
                }
            }
            // Secondary info (only show if there's a value)
            if !props.secondary_value.is_empty() {
                div {
                    class: "flex items-center justify-between text-xs",
                    span {
                        class: "text-muted-foreground",
                        "{props.secondary_label}"
                    }
                    span {
                        class: "font-medium {props.accent_class}",
                        "{props.secondary_value}"
                    }
                }
            } else if !props.secondary_label.is_empty() {
                div {
                    class: "text-xs text-muted-foreground",
                    "{props.secondary_label}"
                }
            }
        }
    }
}

#[component]
fn SummarySkeletonGrid() -> Element {
    rsx! {
        div {
            class: "grid grid-cols-2 lg:grid-cols-4 gap-3",
            for _ in 0..4 {
                SkeletonCard {}
            }
        }
    }
}

#[component]
fn SkeletonCard() -> Element {
    rsx! {
        div {
            class: "\
                animate-pulse rounded-xl \
                border border-border \
                bg-card \
                p-4 space-y-3",
            div { class: "flex items-center justify-between" }
                div { class: "h-3 w-20 rounded bg-muted" }
                div { class: "h-4 w-4 rounded bg-muted" }
            div { class: "h-7 w-16 rounded bg-muted" }
            div { class: "h-3 w-24 rounded bg-muted" }
        }
    }
}

/// Very small helper to keep card numbers readable.
fn format_number(value: i64) -> String {
    match value {
        v if v >= 1_000_000_000 => format!("{:.1}B", v as f64 / 1_000_000_000_f64),
        v if v >= 1_000_000 => format!("{:.1}M", v as f64 / 1_000_000_f64),
        v if v >= 1_000 => format!("{:.1}k", v as f64 / 1_000_f64),
        v => v.to_string(),
    }
}
