mod canvas;
mod compress_tool;
mod crop_tool;
mod resize_tool;
mod rotate_tool;
mod toolbar;

pub use canvas::*;
pub use compress_tool::*;
pub use crop_tool::*;
pub use resize_tool::*;
pub use rotate_tool::*;
pub use toolbar::*;

use crate::store::{use_image_editor, EditorTool};
use crate::ui::custom::AppPortal;
use crate::ui::shadcn::{Button, ButtonVariant};
use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct ImageEditorModalProps {
    pub on_save: EventHandler<web_sys::File>,
}

/// Main image editor modal component
#[component]
pub fn ImageEditorModal(props: ImageEditorModalProps) -> Element {
    let editor = use_image_editor();
    let is_open = *editor.is_open.read();
    let session = editor.current_session.read().clone();
    let active_tool = *editor.active_tool.read();
    let error_message = editor.error_message.read().clone();

    if !is_open {
        return rsx! {};
    }

    let Some(ref session_data) = session else {
        return rsx! {};
    };

    let handle_close = move |_| {
        editor.close_editor();
    };

    let handle_save = move |_| {
        let filename = format!("edited-image-{}.jpg", chrono::Utc::now().timestamp());
        spawn(async move {
            match editor.export_as_file(filename).await {
                Ok(file) => {
                    props.on_save.call(file);
                    editor.close_editor();
                }
                Err(e) => {
                    gloo_console::error!("[ImageEditor] Failed to export:", &e);
                }
            }
        });
    };

    let handle_reset = move |_| {
        editor.reset_to_original();
    };

    rsx! {
        AppPortal {
            class: "z-20",
            // Backdrop
            div {
                class: "fixed inset-0 bg-black/50",
                onclick: handle_close,
            }

            // Modal content
            div {
                class: "fixed inset-4 flex items-center justify-center",
                onclick: move |e| e.stop_propagation(), // Prevent closing when clicking inside

                div {
                    class: "bg-background rounded-lg shadow-2xl max-w-6xl w-full max-h-[90vh] overflow-auto p-6 border border-zinc-200 dark:border-zinc-800",

                    // Error message
                    if let Some(ref error) = error_message {
                        div { class: "mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded text-red-700 dark:text-red-300 text-sm",
                            "{error}"
                        }
                    }

                    // Main content
                    div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6",
                        // Canvas area (2 columns on large screens)
                        div { class: "lg:col-span-2",
                            EditorToolbar {}
                            div { class: "mt-4",
                                EditorCanvas {
                                    blob_url: session_data.current_blob_url.clone(),
                                    width: session_data.width,
                                    height: session_data.height,
                                }
                            }
                        }

                        // Tools panel (1 column on large screens)
                        div { class: "space-y-4",
                            // Show active tool controls
                            match active_tool {
                                EditorTool::Crop => rsx! { CropTool {} },
                                EditorTool::Resize => rsx! { ResizeTool {} },
                                EditorTool::Rotate => rsx! { RotateTool {} },
                                EditorTool::Compress => rsx! { CompressTool {} },
                                EditorTool::None => rsx! {
                                    div { class: "p-4 rounded-lg border border-zinc-200 dark:border-zinc-800 text-center text-muted-foreground",
                                        "Select a tool from the toolbar to start editing"
                                    }
                                },
                            }

                            // Image info
                            div { class: "p-4 rounded-lg border border-zinc-200 dark:border-zinc-800",
                                h4 { class: "text-sm font-semibold mb-2", "Image Info" }
                                div { class: "text-xs text-muted-foreground space-y-1",
                                    div { "Size: {session_data.width} × {session_data.height}px" }
                                }
                            }
                        }
                    }

                    // Footer actions
                    div { class: "flex items-center justify-between pt-4 mt-4 border-t border-zinc-200 dark:border-zinc-800",
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: handle_reset,
                            "Reset to Original"
                        }
                        div { class: "flex gap-2",
                            Button {
                                variant: ButtonVariant::Outline,
                                onclick: handle_close,
                                "Cancel"
                            }
                            Button {
                                onclick: handle_save,
                                "Save"
                            }
                        }
                    }
                }
            }
        }
    }
}
