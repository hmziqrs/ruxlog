use dioxus::prelude::*;

/// Properties for the progress component
#[derive(Props, PartialEq, Clone)]
pub struct ProgressProps {
    /// The current value of the progress bar (0-100)
    #[props(default = 0)]
    pub value: i32,
    /// Additional CSS classes to apply
    #[props(default)]
    pub class: Option<String>,
}

/// Progress component that shows a horizontal progress bar
#[component]
pub fn Progress(props: ProgressProps) -> Element {
    // Clamp value between 0 and 100
    let value = props.value.clamp(0, 100);

    let mut class =
        vec!["bg-primary/20 relative h-2 w-full overflow-hidden rounded-full".to_string()];

    if let Some(custom_class) = &props.class {
        class.push(custom_class.clone());
    }

    // Calculate the transform style based on the value
    let transform_style = format!("transform: translateX(-{}%)", 100 - value);

    rsx! {
        div {
            "data-slot": "progress",
            class: class.join(" "),
            div {
                "data-slot": "progress-indicator",
                class: "bg-primary h-full w-full flex-1 transition-all",
                style: transform_style,
            }
        }
    }
}
