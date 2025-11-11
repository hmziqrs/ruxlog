use dioxus::{logger::tracing, prelude::*};
use hmziq_dioxus_free_icons::{
    icons::ld_icons::{LdCheck, LdChevronDown},
    Icon,
};

use crate::ui::custom::AppPortal;

use super::state::{SelectContext, SelectProps};

#[component]
pub fn Select(props: SelectProps) -> Element {
    let mut state = use_signal(|| SelectContext::new(props.groups.clone(), props.selected));

    rsx! {
        div {
            onkeydown: move |e| {
                e.stop_propagation();
                match e.key() {
                    Key::ArrowDown => {
                        if !state.read().is_open {
                            state.write().open();
                        } else {
                            state.write().next_index();
                        }
                    }
                    Key::ArrowUp => {
                        if !state.read().is_open {
                            state.write().open();
                        } else {
                            state.write().prev_index();
                        }
                    }
                    Key::Enter => {
                        if state.read().is_open {
                            state.write().select_active_index();
                            state.write().close();
                        } else {
                            state.write().open();
                        }
                    }
                    Key::Escape => {
                        state.write().close();
                    }
                    _ => {}
                }
            },
            class: "select-root relative max-w-[15rem]",
            div {
                // variant: ButtonVariant::Outline,
                tabindex: "0",
                class: "flex h-9 w-full items-center justify-between whitespace-nowrap rounded-md border border-input bg-transparent px-3 py-2 text-sm ring-offset-background data-[placeholder]:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-ring disabled:cursor-not-allowed disabled:opacity-50 [&>span]:line-clamp-1",
                onclick: move |_| {
                    state.write().toggle();
                },
                label {
                    if let Some(selected) = state.read().selected.clone() {
                        {selected}
                    } else {
                        {props.placeholder}
                    }
                }
                div { class: "size-5",
                    Icon { icon: LdChevronDown }
                }
            }
            if state.read().is_open {
                AppPortal {
                    onclick: move |_| {
                        state.write().close();
                    },
                }
                div {
                    class: "absolute mt-[1px] bg-background text-popover-foreground z-50 min-w-[15rem] overflow-hidden rounded-lg border animate-in fade-in-0 zoom-in-95",
                    onclick: move |e| {
                        e.stop_propagation();
                    },
                    div { class: "select-content-inner p-1 max-h-[300px] overflow-y-auto",
                        for group in state.read().internal_groups.clone().iter() {
                            div { class: "select-group py-1.5",
                                h3 { class: "select-group-label px-2 text-xs font-semibold text-muted-foreground",
                                    {group.label.clone()}
                                }
                                for item in group.clone().items.into_iter() {
                                    div {
                                        tabindex: "0",
                                        class: format!(
                                            "relative flex cursor-default select-none items-center rounded-sm py-1.5 pl-2 pr-8 text-sm outline-none focus:bg-accent focus:text-accent-foreground hover:bg-accent hover:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50 {}",
                                            if state.read().active_index == item.index {
                                                "bg-accent text-accent-foreground"
                                            } else {
                                                "text-muted-foreground"
                                            },
                                        ),

                                        onclick: move |_| {
                                            tracing::info!("ITEM ONCLICK");
                                            state.write().select(item.value.clone(), item.index);
                                            if let Some(on_select) = props.on_select {
                                                on_select(item.value.clone());
                                            }
                                            state.write().close();
                                        },
                                        // Check mark for selected item
                                        if state.read().selected.as_ref() == Some(&item.value) {
                                            span { class: "absolute right-2 flex size-3.5 items-center justify-center",
                                                Icon {
                                                    icon: LdCheck,
                                                    class: "size-4",
                                                }
                                            }
                                        }
                                        {item.label}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
