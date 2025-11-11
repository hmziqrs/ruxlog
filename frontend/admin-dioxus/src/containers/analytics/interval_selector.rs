use dioxus::prelude::*;

use crate::store::analytics::AnalyticsInterval;

/// Interval selector for analytics charts.
///
/// Renders a compact dropdown/toggle for selecting Hour/Day/Week/Month intervals.
#[derive(Props, Clone, PartialEq)]
pub struct IntervalSelectorProps {
    /// Currently selected interval
    pub current: AnalyticsInterval,
    /// Callback when interval changes
    pub on_change: EventHandler<AnalyticsInterval>,
    /// Optional label (defaults to "Interval:")
    #[props(default = "Interval:".to_string())]
    pub label: String,
}

#[component]
pub fn IntervalSelector(props: IntervalSelectorProps) -> Element {
    let intervals = vec![
        (AnalyticsInterval::Hour, "Hour"),
        (AnalyticsInterval::Day, "Day"),
        (AnalyticsInterval::Week, "Week"),
        (AnalyticsInterval::Month, "Month"),
    ];

    rsx! {
        div {
            class: "flex items-center gap-1.5",
            if !props.label.is_empty() {
                label {
                    class: "text-[9px] font-medium text-zinc-500 dark:text-zinc-400",
                    "{props.label}"
                }
            }
            div {
                class: "inline-flex rounded-md border border-border bg-background p-0.5",
                for (interval, label) in intervals {
                    {
                        let is_active = props.current == interval;
                        let interval_clone = interval.clone();

                        rsx! {
                            button {
                                r#type: "button",
                                onclick: move |_| props.on_change.call(interval_clone),
                                class: if is_active {
                                    "px-2 py-0.5 text-[9px] font-medium rounded \
                                     bg-background text-foreground \
                                     shadow-sm border border-border \
                                     transition-all"
                                } else {
                                    "px-2 py-0.5 text-[9px] font-medium rounded \
                                     text-muted-foreground hover:text-foreground \
                                     hover:bg-background transition-colors"
                                },
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
}
