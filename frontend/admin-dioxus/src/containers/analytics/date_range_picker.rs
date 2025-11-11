use dioxus::prelude::*;

/// Date range picker for analytics filters.
///
/// Renders two date inputs (from/to) and triggers callback on change.
#[derive(Props, Clone, PartialEq)]
pub struct DateRangePickerProps {
    /// Current start date (YYYY-MM-DD format)
    pub date_from: Option<String>,
    /// Current end date (YYYY-MM-DD format)
    pub date_to: Option<String>,
    /// Callback when date range changes
    pub on_change: EventHandler<(Option<String>, Option<String>)>,
}

#[component]
pub fn DateRangePicker(props: DateRangePickerProps) -> Element {
    let from_value = props.date_from.clone().unwrap_or_default();
    let to_value = props.date_to.clone().unwrap_or_default();

    let on_from_change = move |evt: Event<FormData>| {
        let value = evt.value();
        let new_from = if value.is_empty() {
            None
        } else {
            Some(value.clone())
        };
        props.on_change.call((new_from, props.date_to.clone()));
    };

    let on_to_change = move |evt: Event<FormData>| {
        let value = evt.value();
        let new_to = if value.is_empty() {
            None
        } else {
            Some(value.clone())
        };
        props.on_change.call((props.date_from.clone(), new_to));
    };

    rsx! {
        div {
            class: "flex items-center gap-2",
            // From date
            div {
                class: "flex items-center gap-1.5",
                label {
                    class: "text-xs font-medium text-zinc-600 dark:text-zinc-400 whitespace-nowrap",
                    "From"
                }
                input {
                    r#type: "date",
                    value: from_value,
                    oninput: on_from_change,
                    class: "px-2.5 py-1.5 text-xs rounded-md border border-zinc-200 \
                           dark:border-zinc-800 bg-transparent text-foreground \
                           focus:border-ring focus:ring-1 focus:ring-ring/40 \
                           transition-colors",
                }
            }

            // Separator
            span {
                class: "text-zinc-400 dark:text-zinc-600",
                "—"
            }

            // To date
            div {
                class: "flex items-center gap-1.5",
                label {
                    class: "text-xs font-medium text-zinc-600 dark:text-zinc-400 whitespace-nowrap",
                    "To"
                }
                input {
                    r#type: "date",
                    value: to_value,
                    oninput: on_to_change,
                    class: "px-2.5 py-1.5 text-xs rounded-md border border-zinc-200 \
                           dark:border-zinc-800 bg-transparent text-foreground \
                           focus:border-ring focus:ring-1 focus:ring-ring/40 \
                           transition-colors",
                }
            }
        }
    }
}
