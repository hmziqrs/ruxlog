use super::state::*;
use dioxus::{logger::tracing, prelude::*};

#[component]
pub fn Cmdk(props: CommandListProps) -> Element {
    let mut state =
        use_signal(|| CommandContext::new(props.groups.clone(), props.data.clone(), None));

    let read = state.read();
    let groups = read.groups.clone();
    let active_index = read.active_index;

    rsx! {
        div {
            class: "cmdk",
            onkeydown: move |e| {
                let key = e.key();
                match key {
                    Key::ArrowUp => {
                        state.write().set_prev_index();
                    }
                    Key::ArrowDown => {
                        state.write().set_next_index();
                    }
                    _ => {}
                }
            },
            div { class: "cmdk-input",
                input {
                    value: state.read().search.clone(),
                    oninput: move |e| {
                        let value = e.value();
                        tracing::info!("onchange: {}", value);
                        state.write().set_search(value);
                    },
                }
            }
            if read.is_empty {
                div { class: "cmdk-empty", "No results found" }
            }
            div { class: "cmdk-list",
                for group in groups.into_iter() {
                    div { key: "{group.id}", class: "cmdk-group",
                        div { class: "cmdk-group-label", "{group.label}" }
                        for item in group.items.into_iter() {
                            div {
                                key: "{item.value}",
                                class: format!(
                                    "cmdk-group-item {}",
                                    if item.index == active_index { "active bg-red-500" } else { "" },
                                ),
                                "data-value": item.value,
                                {item.label}
                            }
                        }
                    }
                }
            }
        }
    }
}
