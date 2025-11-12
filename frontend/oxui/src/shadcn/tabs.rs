use std::rc::Rc;

use dioxus::{logger::tracing, prelude::*};
use im::HashMap;

#[derive(Props, PartialEq, Clone)]
pub struct TabsProps {
    #[props(default)]
    pub class: Option<String>,

    pub state: Signal<TabsState>,
}

#[derive(PartialEq, Clone)]
pub struct TabItem {
    pub label: String,
    pub disable: bool,
}

impl TabItem {
    pub fn new(label: String, disable: bool) -> Self {
        Self { label, disable }
    }
}

#[derive(PartialEq, Clone)]
pub struct TabsState {
    active_index: usize,
    items: Vec<TabItem>,
}

impl TabsState {
    pub fn new(tabs: Vec<TabItem>, default_index: usize) -> Self {
        Self {
            active_index: default_index,
            items: tabs,
        }
    }

    pub fn set_active_tab(&mut self, index: usize) {
        if index < self.items.len() && !self.items[index].disable {
            self.active_index = index;
        }
    }

    pub fn next_tab(&mut self) {
        let initial_tab = self.active_index.clone();
        loop {
            tracing::info!("next_tab: {}", initial_tab);
            self.active_index = {
                if self.active_index + 1 >= self.items.len() - 1 {
                    0
                } else {
                    self.active_index + 1
                }
            };
            if self.items[self.active_index].disable {
                continue;
            }
            if self.active_index == initial_tab {
                break;
            }
            // self.active_index = self.active_index;
            break;
        }
    }

    pub fn prev_tab(&mut self) {
        let initial_tab = self.active_index.clone();
        loop {
            tracing::info!("prev_tab: {}", initial_tab);
            self.active_index = {
                if self.active_index == 0 {
                    self.items.len() - 1
                } else {
                    self.active_index - 1
                }
            };
            if self.items[self.active_index].disable {
                continue;
            }
            if self.active_index == initial_tab {
                break;
            }
            // self.active_index = self.active_index;
            break;
        }
    }
}

/// A simple tabs component
#[component]
pub fn Tabs(props: TabsProps) -> Element {
    let mut state = props.state;
    let active_index = state.read().active_index;
    let mut refs = use_signal(|| HashMap::<usize, Rc<MountedData>>::new());

    let mut class = vec!["flex flex-col gap-2".to_string()];
    if let Some(custom_class) = props.class.clone() {
        class.push(custom_class);
    }

    rsx! {
        div {
            onkeydown: move |e| {
                let key = e.key();
                match key {
                    Key::ArrowLeft => {
                        state.write().prev_tab();
                    }
                    Key::ArrowRight => {
                        state.write().next_tab();
                    }
                    Key::Enter => {
                        let current_active_index = state.read().active_index;
                        let maybe_mounted = refs.read().get(&current_active_index).cloned();
                        if let Some(mounted) = maybe_mounted {
                            spawn(async move {
                                let _ = mounted.set_focus(true).await;
                            });
                        }
                    }
                    _ => {}
                }
            },
            class: class.join(" "),
            // Tab list
            div { class: "bg-muted text-muted-foreground inline-flex h-9 w-fit items-center justify-center rounded-lg p-[3px]",
                // Tab buttonsc
                for (i , tab) in state.read().items.clone().into_iter().enumerate() {
                    button {
                        tabindex: if !tab.disable { "0" } else { "-1" },
                        onmounted: move |mounted| {
                            refs.write().insert(i, mounted.data());
                        },
                        class: format!(
                            "inline-flex h-[calc(100%-1px)] items-center justify-center rounded-md px-3 py-1 text-sm font-medium transition-all {} {}",
                            if i == active_index { "bg-background shadow-sm text-foreground" } else { "" },
                            if tab.disable { "opacity-50 cursor-not-allowed" } else { "" },
                        ),
                        disabled: tab.disable,
                        onclick: move |_| {
                            tracing::info!("Tab {} clicked", i);
                            state.write().set_active_tab(i);
                        },
                        {tab.label}
                    }
                }
            }
        }
    }
}
