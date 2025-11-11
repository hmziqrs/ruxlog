use crate::ui::shadcn::Button;
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::{icons::ld_icons::LdTag, Icon};

#[derive(Props, PartialEq, Clone)]
pub struct ListEmptyStateProps {
    pub title: String,
    pub description: String,
    pub clear_label: String,
    pub create_label: String,
    pub on_clear: EventHandler<()>,
    pub on_create: EventHandler<()>,
}

/// Standard list empty-state block used across list screens.
#[component]
pub fn ListEmptyState(props: ListEmptyStateProps) -> Element {
    rsx! {
        div { class: "flex flex-col items-center justify-center gap-3 text-center",
            div { class: "flex h-12 w-12 items-center justify-center rounded-full bg-muted",
                div { class: "h-6 w-6 text-muted-foreground", Icon { icon: LdTag {} } }
            }
            div { class: "space-y-1",
                h3 { class: "text-lg font-medium", "{props.title}" }
                p { class: "text-sm text-muted-foreground", "{props.description}" }
            }
            div { class: "flex flex-col items-center gap-2 sm:flex-row",
                Button { variant: crate::ui::shadcn::ButtonVariant::Outline, onclick: move |_| { props.on_clear.call(()); }, "{props.clear_label}" }
                Button { onclick: move |_| { props.on_create.call(()); }, "{props.create_label}" }
            }
        }
    }
}
