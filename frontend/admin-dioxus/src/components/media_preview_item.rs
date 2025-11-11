use crate::store::Media;
use crate::ui::shadcn::{Button, ButtonVariant};
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::{
    icons::ld_icons::{LdPencil, LdTrash2},
    Icon,
};

#[component]
pub fn MediaPreviewItem(
    /// The media object to display
    media: Media,
    /// Optional callback when user clicks remove
    #[props(default)]
    on_remove: Option<EventHandler<i32>>,
    /// Optional callback when user clicks edit
    #[props(default)]
    on_edit: Option<EventHandler<String>>,
    /// Alternative text for the image
    #[props(default = "Preview".to_string())]
    alt: String,
) -> Element {
    let is_image = media.mime_type.starts_with("image/");
    let filename = media
        .object_key
        .split('/')
        .last()
        .unwrap_or("Unknown")
        .to_string();

    rsx! {
        div { class: "group relative rounded-lg border border-border/60 bg-card overflow-hidden transition-all hover:border-border hover:shadow-sm",
            // Preview section
            div { class: "relative aspect-video bg-muted/30 flex items-center justify-center overflow-hidden",
                if is_image {
                    img {
                        src: "{media.file_url}",
                        alt: "{alt}",
                        class: "w-full h-full object-cover"
                    }
                } else {
                    div { class: "flex flex-col items-center justify-center gap-2 text-muted-foreground",
                        div { class: "text-4xl", "📄" }
                        p { class: "text-sm", "{filename}" }
                    }
                }

                // Overlay with actions (shown on hover)
                div { class: "absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-2",
                    if let Some(on_edit_handler) = on_edit {
                        Button {
                            variant: ButtonVariant::Secondary,
                            class: "h-8 px-3",
                            onclick: move |_| {
                                on_edit_handler.call(media.file_url.clone());
                            },
                            Icon {
                                icon: LdPencil,
                                class: "h-4 w-4 mr-1.5"
                            }
                            "Edit"
                        }
                    }
                    if let Some(on_remove_handler) = on_remove {
                        Button {
                            variant: ButtonVariant::Destructive,
                            class: "h-8 px-3",
                            onclick: move |_| {
                                on_remove_handler.call(media.id);
                            },
                            Icon {
                                icon: LdTrash2,
                                class: "h-4 w-4 mr-1.5"
                            }
                            "Remove"
                        }
                    }
                }
            }

            // Info section
            div { class: "px-3 py-2.5 space-y-1",
                p { class: "text-sm font-medium text-foreground truncate", "{filename}" }
                div { class: "flex items-center gap-2 text-xs text-muted-foreground",
                    span { "{format_file_size(media.size)}" }
                    if let (Some(w), Some(h)) = (media.width, media.height) {
                        span { "•" }
                        span { "{w} × {h}" }
                    }
                }
            }
        }
    }
}

fn format_file_size(bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
