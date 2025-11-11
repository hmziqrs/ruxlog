use crate::store::use_image_editor;
use crate::ui::shadcn::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;

/// Rotate tool controls
#[component]
pub fn RotateTool() -> Element {
    let editor = use_image_editor();
    let is_processing = *editor.is_processing.read();

    let handle_apply = move |_| {
        spawn(async move {
            let _ = editor.apply_rotate().await;
        });
    };

    let quick_rotate = move |angle: i32| {
        move |_| {
            editor.rotate_params.write().angle = angle;
        }
    };

    rsx! {
        div {
            class: "space-y-4 p-4 rounded-lg border border-zinc-200 dark:border-zinc-800",

            h3 { class: "text-sm font-semibold mb-2", "Rotate Settings" }

            // Quick rotation buttons
            div { class: "grid grid-cols-3 gap-2",
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    onclick: quick_rotate(90),
                    "90°"
                }
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    onclick: quick_rotate(180),
                    "180°"
                }
                Button {
                    variant: ButtonVariant::Outline,
                    size: ButtonSize::Sm,
                    onclick: quick_rotate(270),
                    "270°"
                }
            }

            // Custom angle input
            div {
                label { class: "block text-xs text-muted-foreground mb-1", "Custom Angle (degrees)" }
                input {
                    r#type: "number",
                    class: "w-full px-2 py-1 text-sm border border-zinc-200 dark:border-zinc-800 rounded",
                    value: "{editor.rotate_params.read().angle}",
                    min: -360,
                    max: 360,
                    oninput: move |e| {
                        if let Ok(val) = e.value().parse::<i32>() {
                            editor.rotate_params.write().angle = val;
                        }
                    }
                }
            }

            Button {
                onclick: handle_apply,
                disabled: is_processing,
                class: "w-full",
                if is_processing { "Applying..." } else { "Apply Rotation" }
            }
        }
    }
}
