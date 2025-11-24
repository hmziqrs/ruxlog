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
                accent_class: "text-emerald-500",
            }
            // Views - Key engagement metric
            SummaryCard {
                label: "Page views",
                primary_value: format_number(summary.posts.views_in_period),
                secondary_label: "This period",
                secondary_value: "".to_string(),
                accent_class: "text-sky-500",
            }
            // Comments - Engagement
            SummaryCard {
                label: "Comments",
                primary_value: format_number(summary.engagement.comments_in_period),
                secondary_label: "This period",
                secondary_value: "".to_string(),
                accent_class: "text-violet-500",
            }
            // Media - Content assets
            SummaryCard {
                label: "Media files",
                primary_value: format_number(summary.media.total_files),
                secondary_label: "New uploads",
                secondary_value: format_number(summary.media.uploads_in_period),
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
    accent_class: &'static str,
}

#[component]
fn SummaryCard(props: SummaryCardProps) -> Element {
    rsx! {
        div {
            class: "flex flex-col gap-2 rounded-lg border border-border p-4",
            // Label
            span {
                class: "text-xs font-medium text-muted-foreground",
                "{props.label}"
            }
            // Primary value
            span {
                class: "text-2xl font-semibold tracking-tight text-foreground",
                "{props.primary_value}"
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
                span {
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
            class: "rounded-lg border border-border p-4 space-y-2",
            div { class: "h-3 w-20 rounded bg-muted animate-pulse" }
            div { class: "h-7 w-16 rounded bg-muted animate-pulse" }
            div { class: "h-3 w-24 rounded bg-muted animate-pulse" }
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
