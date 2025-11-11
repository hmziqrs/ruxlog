use dioxus::prelude::*;
use std::collections::HashMap;

/// A component for rendering a button with various styles.
///
/// This is a port of the ShadcnUI button component from React to Dioxus.
/// It supports different variants and sizes, similar to the original component.
#[derive(Props, PartialEq, Clone)]
pub struct ButtonProps {
    /// The content to be displayed inside the button
    children: Element,

    /// Additional CSS classes to apply to the button
    #[props(default)]
    class: Option<String>,

    /// The variant style of the button
    #[props(default = ButtonVariant::Default)]
    variant: ButtonVariant,

    /// The size of the button
    #[props(default = ButtonSize::Default)]
    size: ButtonSize,

    /// Whether the button is disabled
    #[props(default = false)]
    disabled: bool,

    /// Optional click handler for the button
    #[props(default)]
    onclick: Option<EventHandler<MouseEvent>>,

    /// HTML type attribute for the button
    #[props(default = "button".to_string())]
    r#type: String,

    /// Optional aria-label for accessibility
    #[props(default)]
    aria_label: Option<String>,

    /// Optional data attributes to add to the button
    #[props(default)]
    data_attributes: Option<HashMap<String, String>>,

    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
}

/// The different style variants a button can have
#[derive(PartialEq, Clone, Copy)]
pub enum ButtonVariant {
    Default,
    Destructive,
    Outline,
    Secondary,
    Ghost,
    Link,
}

/// The different size options for buttons
#[derive(PartialEq, Clone, Copy)]
pub enum ButtonSize {
    Default,
    Sm,
    Lg,
    Icon,
}

/// Get the CSS class for a button variant
fn get_variant_class(variant: ButtonVariant) -> &'static str {
    match variant {
        ButtonVariant::Default => "bg-primary text-primary-foreground hover:bg-primary/90",
        ButtonVariant::Destructive => "bg-destructive text-white hover:bg-destructive/90 focus-visible:ring-destructive/20 dark:focus-visible:ring-destructive/40 dark:bg-destructive/60",
        ButtonVariant::Outline => "border bg-background hover:bg-accent hover:text-accent-foreground dark:bg-input/30 dark:border-input dark:hover:bg-input/50",
        ButtonVariant::Secondary => "bg-secondary text-secondary-foreground hover:bg-secondary/80",
        ButtonVariant::Ghost => "hover:bg-accent hover:text-accent-foreground dark:hover:bg-accent/50",
        ButtonVariant::Link => "text-primary underline-offset-4 hover:underline",
    }
}

/// Get the CSS class for a button size
fn get_size_class(size: ButtonSize) -> &'static str {
    match size {
        ButtonSize::Default => "h-10 px-4 py-2 has-[>svg]:px-3",
        ButtonSize::Sm => "h-9 rounded-md gap-1.5 px-3 has-[>svg]:px-2.5",
        ButtonSize::Lg => "h-12 rounded-md px-6 has-[>svg]:px-4",
        ButtonSize::Icon => "size-12",
    }
}

/// Button component
#[component]
pub fn Button(props: ButtonProps) -> Element {
    // Combine all CSS classes
    let mut class = vec![
        "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md text-sm font-medium transition-all disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4 shrink-0 [&_svg]:shrink-0 outline-none focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-[3px] aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive".to_string(),
        get_variant_class(props.variant).to_string(),
        get_size_class(props.size).to_string(),
    ];

    // Add custom class if provided
    if let Some(custom_class) = &props.class {
        class.push(custom_class.clone());
    }

    rsx! {
        button {
            class: class.join(" "),
            r#type: props.r#type,
            disabled: props.disabled,
            onclick: move |e| {
                if let Some(handler) = &props.onclick {
                    handler.call(e);
                }
            },
            aria_label: props.aria_label,
            // Apply any data attributes
            // {data_attrs.iter().map(|(key, value)| {
            //         data: { key: value }
            // })},
            // data_slot: "button",
            ..props.attributes,
            {props.children}
        }
    }
}
