use crate::store::use_image_editor;
use crate::ui::shadcn::Button;
use dioxus::prelude::*;

/// Resize tool controls
#[component]
pub fn ResizeTool() -> Element {
    let editor = use_image_editor();
    let is_processing = *editor.is_processing.read();

    let handle_apply = move |_| {
        spawn(async move {
            let _ = editor.apply_resize().await;
        });
    };

    let handle_maintain_aspect = move |_| {
        let mut params = editor.resize_params.write();
        params.maintain_aspect_ratio = !params.maintain_aspect_ratio;
    };

    rsx! {
        div {
            class: "space-y-4 p-4 rounded-lg border border-zinc-200 dark:border-zinc-800",

            h3 { class: "text-sm font-semibold mb-2", "Resize Settings" }

            div { class: "space-y-3",
                // Width
                div {
                    label { class: "block text-xs text-muted-foreground mb-1", "Width (px)" }
                    input {
                        r#type: "number",
                        class: "w-full px-2 py-1 text-sm border border-zinc-200 dark:border-zinc-800 rounded",
                        value: "{editor.resize_params.read().width}",
                        min: 1,
                        oninput: move |e| {
                            if let Ok(val) = e.value().parse::<u32>() {
                                editor.resize_params.write().width = val;
                            }
                        }
                    }
                }

                // Height
                div {
                    label { class: "block text-xs text-muted-foreground mb-1", "Height (px)" }
                    input {
                        r#type: "number",
                        class: "w-full px-2 py-1 text-sm border border-zinc-200 dark:border-zinc-800 rounded",
                        value: "{editor.resize_params.read().height}",
                        min: 1,
                        oninput: move |e| {
                            if let Ok(val) = e.value().parse::<u32>() {
                                editor.resize_params.write().height = val;
                            }
                        }
                    }
                }

                // Maintain aspect ratio checkbox
                div { class: "flex items-center gap-2",
                    input {
                        r#type: "checkbox",
                        id: "maintain-aspect",
                        checked: editor.resize_params.read().maintain_aspect_ratio,
                        onchange: handle_maintain_aspect,
                    }
                    label {
                        r#for: "maintain-aspect",
                        class: "text-sm text-muted-foreground",
                        "Maintain aspect ratio"
                    }
                }
            }

            Button {
                onclick: handle_apply,
                disabled: is_processing,
                class: "w-full",
                if is_processing { "Applying..." } else { "Apply Resize" }
            }
        }
    }
}
