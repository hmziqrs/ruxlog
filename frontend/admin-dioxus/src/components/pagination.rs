use crate::store::PaginatedList;
use crate::ui::shadcn::{Button, ButtonVariant};
use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct PaginationProps<T: Clone + PartialEq + 'static> {
    /// Current page data; when None, component renders blank label and disables nav
    pub page: Option<PaginatedList<T>>,
    /// Disable all interactions (loading state)
    #[props(default = false)]
    pub disabled: bool,
    /// Callback when previous is clicked
    pub on_prev: EventHandler<()>,
    /// Callback when next is clicked
    pub on_next: EventHandler<()>,
}

#[component]
pub fn Pagination<T: Clone + PartialEq + 'static>(props: PaginationProps<T>) -> Element {
    let (label, prev_disabled, next_disabled) = if let Some(p) = props.page.as_ref() {
        (
            format!("Page {} • {} items", p.page, p.total),
            !p.has_previous_page(),
            !p.has_next_page(),
        )
    } else {
        (String::new(), true, true)
    };

    let prev_is_disabled = props.disabled || prev_disabled;
    let next_is_disabled = props.disabled || next_disabled;

    rsx! {
        div { class: "flex items-center justify-between border-t border-border/60 p-3 text-sm text-muted-foreground",
            div { class: "hidden md:block", "{label}" }
            div { class: "flex items-center gap-2 ml-auto",
                Button { variant: ButtonVariant::Outline, disabled: prev_is_disabled, onclick: move |_| { props.on_prev.call(()); }, "Previous" }
                Button { disabled: next_is_disabled, onclick: move |_| { props.on_next.call(()); }, "Next" }
            }
        }
    }
}
