use dioxus::prelude::*;
use hmziq_dioxus_free_icons::{icons::ld_icons::LdCheck, Icon};

/// Properties for the Checkbox component
#[derive(Props, PartialEq, Clone)]
pub struct CheckboxProps {
    /// Additional CSS classes to apply to the checkbox
    #[props(default)]
    pub class: Option<String>,

    /// Whether the checkbox is checked
    #[props(default)]
    pub checked: bool,

    /// Whether the checkbox is disabled
    #[props(default)]
    pub disabled: bool,

    /// Callback when checkbox value changes
    #[props(default)]
    pub onchange: Option<EventHandler<bool>>,

    /// Whether the input is in an error state
    #[props(default)]
    pub invalid: bool,
}

/// Checkbox component
#[component]
pub fn Checkbox(props: CheckboxProps) -> Element {
    let mut class = vec!["peer border-input dark:bg-input/30 data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground dark:data-[state=checked]:bg-primary data-[state=checked]:border-primary focus-visible:border-ring focus-visible:ring-ring/50 size-4 shrink-0 rounded-[4px] border outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50".to_string()];

    if props.invalid {
        class.extend_from_slice(&[
            "aria-invalid:ring-destructive/20".to_string(),
            "dark:aria-invalid:ring-destructive/40".to_string(),
            "aria-invalid:border-destructive".to_string(),
        ]);
    }

    if let Some(custom_class) = &props.class {
        class.push(custom_class.clone());
    }

    rsx! {
        div {
            "data-slot": "checkbox",
            "data-state": if props.checked { "checked" } else { "unchecked" },
            "aria-invalid": props.invalid.to_string(),
            class: class.join(" "),
            onclick: move |_| {
                if !props.disabled {
                    if let Some(handler) = &props.onchange {
                        handler.call(!props.checked);
                    }
                }
            },
            if props.checked {
                div {
                    "data-slot": "checkbox-indicator",
                    class: "flex items-center justify-center text-current",
                    div { class: "w-3.5 h-3.5",
                        Icon { icon: LdCheck {} }
                    }
                }
            }
        }
    }
}
