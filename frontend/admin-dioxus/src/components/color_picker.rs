use dioxus::prelude::*;
use hmziq_dioxus_free_icons::{icons::ld_icons::LdCheck, Icon};

const PREDEFINED_COLORS: [&str; 18] = [
    "#ef4444", // red
    "#f97316", // orange
    "#f59e0b", // amber
    "#eab308", // yellow
    "#84cc16", // lime
    "#22c55e", // green
    "#10b981", // emerald
    "#14b8a6", // teal
    "#06b6d4", // cyan
    "#0ea5e9", // sky
    "#3b82f6", // blue
    "#6366f1", // indigo
    "#8b5cf6", // violet
    "#a855f7", // purple
    "#d946ef", // fuchsia
    "#ec4899", // pink
    "#f43f5e", // rose
    "#64748b", // slate
];

#[derive(Props, PartialEq, Clone)]
pub struct ColorPickerProps {
    pub value: String,
    pub onchange: EventHandler<String>,
    #[props(default)]
    pub class: Option<String>,
}

#[component]
pub fn ColorPicker(props: ColorPickerProps) -> Element {
    let mut custom_color = use_signal(|| props.value.clone());

    // Keep internal state in sync if the parent updates `value`
    let value_prop = props.value.clone();
    use_effect(move || {
        let current = custom_color();
        if current != value_prop {
            custom_color.set(value_prop.clone());
        }
    });

    let container_class = {
        let mut base = String::from("space-y-3");
        if let Some(extra) = &props.class {
            if !extra.is_empty() {
                base.push(' ');
                base.push_str(extra);
            }
        }
        base
    };

    rsx! {
        div { class: container_class,
            // Preset colors grid
            div { class: "flex flex-wrap gap-2",
                for color in PREDEFINED_COLORS.iter() {
                    button {
                        r#type: "button",
                        class: format_args!(
                            "w-8 h-8 rounded-full flex items-center justify-center transition-transform{}",
                            if props.value == *color { " ring-2 ring-zinc-400 dark:ring-zinc-300 scale-110" } else { "" }
                        ),
                        style: format!("background-color: {};", color),
                        onclick: move |_| {
                            let s = (*color).to_string();
                            custom_color.set(s.clone());
                            props.onchange.call(s);
                        },
                        if props.value == *color {
                            div { class: "w-4 h-4 text-white",
                                Icon { icon: LdCheck }
                            }
                        }
                    }
                }
            }

            // Custom color section
            div { class: "flex items-center gap-3",
                // Preview swatch
                {
                    let preview_style = format!(
                        "background-color: {};",
                        custom_color()
                    );
                    rsx! { div { class: "w-10 h-10 rounded-full border border-zinc-200 dark:border-zinc-700", style: preview_style } }
                }

                // Hex input
                input {
                    r#type: "text",
                    value: custom_color(),
                    placeholder: "#hex",
                    class: "flex-1 h-9 px-3 py-2 rounded-md border border-zinc-200 dark:border-zinc-800 bg-white dark:bg-zinc-900 text-sm",
                    oninput: move |evt| {
                        let val = evt.value();
                        custom_color.set(val.clone());
                        props.onchange.call(val);
                    }
                }

                // Native color input
                input {
                    r#type: "color",
                    value: custom_color(),
                    class: "w-10 h-10 rounded-md border border-zinc-200 dark:border-zinc-800 bg-white dark:bg-zinc-900 cursor-pointer",
                    oninput: move |evt| {
                        let val = evt.value();
                        custom_color.set(val.clone());
                        props.onchange.call(val);
                    }
                }
            }
        }
    }
}
