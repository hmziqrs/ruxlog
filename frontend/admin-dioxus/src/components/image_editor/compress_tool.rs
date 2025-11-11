use crate::store::use_image_editor;
use crate::ui::shadcn::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;

/// Format bytes to human readable format
fn format_bytes(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = KB * 1024;

    if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Compress tool controls
#[component]
pub fn CompressTool() -> Element {
    let editor = use_image_editor();
    let is_processing = *editor.is_processing.read();
    let compression_savings = editor.compression_savings.read().clone();

    let handle_apply = move |_| {
        spawn(async move {
            let _ = editor.apply_compress().await;
        });
    };

    rsx! {
        div {
            class: "space-y-4 p-4 rounded-lg border border-zinc-200 dark:border-zinc-800",

            h3 { class: "text-sm font-semibold mb-2", "Compression Settings" }

            div { class: "space-y-3",
                // Quality slider
                div {
                    label { class: "block text-xs text-muted-foreground mb-1",
                        "Quality: {editor.compress_params.read().quality}%"
                    }
                    input {
                        r#type: "range",
                        class: "w-full accent-primary",
                        value: "{editor.compress_params.read().quality}",
                        min: 1,
                        max: 100,
                        oninput: move |e| {
                            if let Ok(val) = e.value().parse::<u8>() {
                                editor.compress_params.write().quality = val;
                            }
                        }
                    }
                }

                // Quality presets
                div { class: "grid grid-cols-3 gap-2",
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        onclick: move |_| editor.compress_params.write().quality = 60,
                        "Low (60%)"
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        onclick: move |_| editor.compress_params.write().quality = 85,
                        "Medium (85%)"
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Sm,
                        onclick: move |_| editor.compress_params.write().quality = 95,
                        "High (95%)"
                    }
                }

                // Size savings display
                if let Some((original, current)) = compression_savings {
                    div { class: "p-3 rounded border border-zinc-200 dark:border-zinc-800",
                        div { class: "space-y-1.5",
                            div { class: "flex items-center justify-between text-xs",
                                span { class: "text-muted-foreground", "Original:" }
                                span { class: "font-mono font-medium", "{format_bytes(original)}" }
                            }
                            div { class: "flex items-center justify-between text-xs",
                                span { class: "text-muted-foreground", "Compressed:" }
                                span { class: "font-mono font-medium", "{format_bytes(current)}" }
                            }
                            div { class: "h-px bg-zinc-200 dark:bg-zinc-800 my-1" }
                            div { class: "flex items-center justify-between text-xs",
                                span { class: "text-muted-foreground", "Savings:" }
                                {
                                    let saved = original.saturating_sub(current);
                                    let percent = if original > 0 {
                                        (saved as f64 / original as f64 * 100.0) as i32
                                    } else {
                                        0
                                    };
                                    rsx! {
                                        span { class: "font-mono font-semibold text-green-600 dark:text-green-500",
                                            "{format_bytes(saved)} ({percent}%)"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Helper text
                p { class: "text-xs text-muted-foreground",
                    "Lower quality = smaller file size. Higher quality = better image but larger file."
                }
            }

            Button {
                onclick: handle_apply,
                disabled: is_processing,
                class: "w-full",
                if is_processing { "Applying..." } else { "Apply Compression" }
            }
        }
    }
}
