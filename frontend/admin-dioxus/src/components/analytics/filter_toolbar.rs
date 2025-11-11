use dioxus::prelude::*;

use crate::store::analytics::use_analytics_filters;

use super::date_range_picker::DateRangePicker;
use super::period_preset_toggle::PeriodPresetToggle;

/// Analytics filter toolbar component.
///
/// Sticky toolbar with:
/// - Date range picker (global)
/// - Period preset buttons (7d/30d/90d)
///
/// Automatically updates global filter state and triggers refetch callback.
#[derive(Props, Clone, PartialEq)]
pub struct AnalyticsFilterToolbarProps {
    /// Callback to trigger data refetch when filters change
    pub on_filter_change: EventHandler<()>,
}

#[component]
pub fn AnalyticsFilterToolbar(props: AnalyticsFilterToolbarProps) -> Element {
    let filters = use_analytics_filters();

    // Read current filter state
    let date_from = filters.date_from.read().clone();
    let date_to = filters.date_to.read().clone();
    let active_period = filters.period_preset.read().clone();

    // Handle date range change
    let on_date_change = move |(from, to): (Option<String>, Option<String>)| {
        filters.set_date_range(from, to);
        props.on_filter_change.call(());
    };

    // Handle period preset change
    let on_period_change = move |period: String| {
        filters.set_period_preset(&period);
        props.on_filter_change.call(());
    };

    rsx! {
        div {
            class: "sticky top-0 backdrop-blur-sm bg-background \
                   border-b border-border/60 transition-colors",
            div {
                class: "container mx-auto px-4 py-3",
                div {
                    class: "flex flex-col sm:flex-row items-start sm:items-center \
                           justify-between gap-3",

                    // Left side: Date range picker
                    DateRangePicker {
                        date_from: date_from,
                        date_to: date_to,
                        on_change: on_date_change,
                    }

                    // Right side: Period presets
                    PeriodPresetToggle {
                        active_period: active_period,
                        on_change: on_period_change,
                    }
                }
            }
        }
    }
}
