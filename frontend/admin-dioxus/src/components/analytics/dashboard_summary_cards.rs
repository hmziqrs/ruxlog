use dioxus::prelude::*;

use crate::store::{AnalyticsEnvelopeResponse, DashboardSummaryData, StateFrame, StateFrameStatus};

/// Props:
/// - `frame`: state frame for the dashboard summary analytics request
/// - `title`: optional header title
/// - `description`: optional header description
#[derive(Props, PartialEq, Clone)]
pub struct DashboardSummaryCardsProps {
    pub frame: StateFrame<
        AnalyticsEnvelopeResponse<DashboardSummaryData>,
        crate::store::DashboardSummaryRequest,
    >,
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
            class: "grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-3",
            // Users
            SummaryCard {
                label: "Total users",
                primary_value: format_number(summary.users.total),
                primary_trend_label: "New in period",
                primary_trend_value: format_number(summary.users.new_in_period),
                accent_class: "text-sky-500",
            }
            // Content
            SummaryCard {
                label: "Published posts",
                primary_value: format_number(summary.posts.published),
                primary_trend_label: "Drafts",
                primary_trend_value: format_number(summary.posts.drafts),
                accent_class: "text-emerald-500",
            }
            SummaryCard {
                label: "Views (period)",
                primary_value: format_number(summary.posts.views_in_period),
                primary_trend_label: "Comments",
                primary_trend_value: format_number(summary.engagement.comments_in_period),
                accent_class: "text-indigo-500",
            }
            // Engagement / Newsletter
            SummaryCard {
                label: "Newsletter confirmed",
                primary_value: format_number(summary.engagement.newsletter_confirmed),
                primary_trend_label: "Uploads (period)",
                primary_trend_value: format_number(summary.media.uploads_in_period),
                accent_class: "text-amber-500",
            }
            // Media
            SummaryCard {
                label: "Media files",
                primary_value: format_number(summary.media.total_files),
                primary_trend_label: "New uploads",
                primary_trend_value: format_number(summary.media.uploads_in_period),
                accent_class: "text-rose-500",
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct SummaryCardProps {
    label: &'static str,
    primary_value: String,
    primary_trend_label: &'static str,
    primary_trend_value: String,
    accent_class: &'static str,
}

#[component]
fn SummaryCard(props: SummaryCardProps) -> Element {
    rsx! {
        div {
            class: "\
                group relative flex flex-col justify-between gap-1.5 \
                rounded-xl border border-border \
                bg-background \
                px-3 py-2.5 shadow-none \
                transition-all duration-200 \
                hover:border-muted",
            div {
                class: "flex items-center justify-between gap-2",
                span {
                    class: "text-[10px] font-medium uppercase tracking-wide text-zinc-500 dark:text-zinc-400",
                    "{props.label}"
                }
                span {
                    class: "text-[8px] text-zinc-400 dark:text-zinc-500 group-hover:text-zinc-500 dark:group-hover:text-zinc-400 transition-colors",
                    "Summary"
                }
            }
            div {
                class: "flex items-baseline gap-1.5",
                span {
                    class: "text-xl font-semibold text-zinc-900 dark:text-zinc-50",
                    "{props.primary_value}"
                }
            }
            div {
                class: "flex items-baseline justify-between gap-2 mt-0.5",
                span {
                    class: "text-[9px] text-zinc-500 dark:text-zinc-400",
                    "{props.primary_trend_label}"
                }
                span {
                    class: "text-[9px] font-medium {props.accent_class}",
                    "{props.primary_trend_value}"
                }
            }
        }
    }
}

#[component]
fn SummarySkeletonGrid() -> Element {
    rsx! {
        div {
            class: "grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-3",
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
                bg-background \
                px-3 py-2.5 space-y-2",
            div { class: "h-2 w-14 rounded-full bg-muted" }
            div { class: "h-4 w-16 rounded-full bg-muted" }
            div { class: "h-2 w-20 rounded-full bg-muted" }
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
