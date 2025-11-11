use dioxus::prelude::*;

/// A reusable two-column form loading skeleton matching the tag edit screen layout.
#[component]
pub fn FormTwoColumnSkeleton() -> Element {
    rsx! {
        div { class: "grid grid-cols-1 gap-10 lg:grid-cols-3",
            div { class: "lg:col-span-2 space-y-8",
                div { class: "rounded-xl border border-zinc-200 dark:border-zinc-800 bg-white dark:bg-zinc-900/60 shadow-sm p-6",
                    div { class: "h-5 w-40 rounded bg-muted animate-pulse" }
                    div { class: "mt-4 space-y-3",
                        div { class: "h-9 w-full rounded bg-muted animate-pulse" }
                        div { class: "h-9 w-1/2 rounded bg-muted animate-pulse" }
                        div { class: "h-32 w-full rounded bg-muted animate-pulse" }
                    }
                }
            }
            div { class: "space-y-8",
                div { class: "rounded-xl border border-zinc-200 dark:border-zinc-800 bg-white dark:bg-zinc-900/60 shadow-sm p-6 space-y-4",
                    div { class: "h-5 w-24 rounded bg-muted animate-pulse" }
                    div { class: "h-9 w-full rounded bg-muted animate-pulse" }
                    div { class: "h-9 w-full rounded bg-muted animate-pulse" }
                }
                div { class: "rounded-xl border border-zinc-200 dark:border-zinc-800 bg-white dark:bg-zinc-900/60 shadow-sm p-6 space-y-4",
                    div { class: "h-5 w-24 rounded bg-muted animate-pulse" }
                    div { class: "h-9 w-full rounded bg-muted animate-pulse" }
                }
                div { class: "flex gap-3",
                    div { class: "h-9 w-24 rounded bg-muted animate-pulse" }
                    div { class: "h-9 w-28 rounded bg-muted animate-pulse" }
                }
            }
        }
    }
}
