use crate::components::PageHeader;
use crate::router::Route;
use crate::store::{use_post, EditorJsBlock, PostContent};
use crate::ui::shadcn::{Button, ButtonVariant};
use dioxus::prelude::*;

// ============================================================================
// EditorJS Block Renderers
// ============================================================================

fn render_header_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::Header { data, .. } = block {
        let level = data.level;
        let text = data.text.clone();

        match level {
            1 => rsx! { h1 { class: "text-4xl font-bold mb-6", "{text}" } },
            2 => rsx! { h2 { class: "text-3xl font-bold mb-5", "{text}" } },
            3 => rsx! { h3 { class: "text-2xl font-bold mb-4", "{text}" } },
            4 => rsx! { h4 { class: "text-xl font-bold mb-3", "{text}" } },
            5 => rsx! { h5 { class: "text-lg font-bold mb-2", "{text}" } },
            6 => rsx! { h6 { class: "text-base font-bold mb-2", "{text}" } },
            _ => rsx! { h1 { class: "text-4xl font-bold mb-6", "{text}" } },
        }
    } else {
        rsx! {}
    }
}

fn render_paragraph_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::Paragraph { data, .. } = block {
        // Simple HTML entity decode for &nbsp; etc.
        let text = data
            .text
            .replace("&nbsp;", " ")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&amp;", "&");

        rsx! {
            p { class: "mb-4 leading-7", dangerous_inner_html: "{text}" }
        }
    } else {
        rsx! {}
    }
}

fn render_code_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::Code { data, .. } = block {
        let code = data.code.clone();
        rsx! {
            div { class: "my-6 rounded-lg overflow-hidden border bg-muted/50",
                pre { class: "p-4 overflow-x-auto text-sm",
                    code { class: "font-mono", "{code}" }
                }
            }
        }
    } else {
        rsx! {}
    }
}

fn render_quote_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::Quote { data, .. } = &block {
        let alignment = match data.alignment.as_str() {
            "center" => "text-center",
            "right" => "text-right",
            _ => "text-left",
        };
        let text = data.text.clone();
        let caption = data.caption.clone();

        rsx! {
            blockquote { class: "my-6 pl-4 border-l-4 border-muted italic",
                p { class: "text-lg mb-2 {alignment}", "{text}" }
                if let Some(caption) = caption {
                    footer { class: "text-sm text-muted-foreground {alignment}", "{caption}" }
                }
            }
        }
    } else {
        rsx! {}
    }
}

fn render_alert_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::Alert { data, .. } = &block {
        let alert_style = match data.alert_type.as_str() {
            "success" => "bg-green-50 border-green-200 text-green-800 dark:bg-green-950/20 dark:border-green-800 dark:text-green-300",
            "warning" => "bg-yellow-50 border-yellow-200 text-yellow-800 dark:bg-yellow-950/20 dark:border-yellow-800 dark:text-yellow-300",
            "error" => "bg-red-50 border-red-200 text-red-800 dark:bg-red-950/20 dark:border-red-800 dark:text-red-300",
            _ => "bg-blue-50 border-blue-200 text-blue-800 dark:bg-blue-950/20 dark:border-blue-800 dark:text-blue-300", // info
        };

        let alignment = match data.align.as_str() {
            "center" => "text-center",
            "right" => "text-right",
            _ => "text-left",
        };
        let message = data.message.clone();

        rsx! {
            div { class: "my-6 p-4 rounded-lg border {alert_style} {alignment}",
                p { "{message}" }
            }
        }
    } else {
        rsx! {}
    }
}

fn render_checklist_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::Checklist { data, .. } = &block {
        rsx! {
            ul { class: "my-6 space-y-2",
                for item in &data.items {
                    li { class: "flex items-start gap-2",
                        input {
                            type: "checkbox",
                            checked: item.checked,
                            class: "mt-1",
                            readonly: true
                        }
                        span {
                            class: if item.checked { "flex-1 line-through text-muted-foreground" } else { "flex-1" },
                            "{item.text}"
                        }
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}

fn render_list_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::List { data, .. } = block {
        let list_items = data.items.clone();
        let is_ordered = data.style == "ordered";

        if is_ordered {
            rsx! {
                ol { class: "my-6 ml-6 list-decimal space-y-2",
                    for item in list_items {
                        li { class: "leading-7", "{item}" }
                    }
                }
            }
        } else {
            rsx! {
                ul { class: "my-6 ml-6 list-disc space-y-2",
                    for item in list_items {
                        li { class: "leading-7", "{item}" }
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}

fn render_delimiter_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::Delimiter { .. } = block {
        rsx! {
            div { class: "my-8 flex items-center justify-center",
                hr { class: "w-16 border-t-2 border-muted" }
            }
        }
    } else {
        rsx! {}
    }
}

fn render_image_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::Image { data, .. } = block {
        let url = data.file.url.clone();
        let caption = &data.caption;

        rsx! {
            div { class: "my-6",
                img {
                    src: "{url}",
                    alt: caption.as_deref().unwrap_or(""),
                    class: "w-full h-auto rounded-lg"
                }
                if let Some(ref caption) = data.caption {
                    p { class: "mt-2 text-sm text-muted-foreground text-center italic", "{caption}" }
                }
            }
        }
    } else {
        rsx! {}
    }
}

fn render_embed_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::Embed { data, .. } = block {
        let source = data.source.clone();

        rsx! {
            div { class: "my-6 rounded-lg overflow-hidden",
                iframe {
                    src: "{source}",
                    class: "w-full",
                    style: format!("height: {}px;", data.height.unwrap_or(450)),
                    frame_border: "0",
                    allowfullscreen: "true"
                }
                if let Some(ref caption) = data.caption {
                    p { class: "mt-2 text-sm text-muted-foreground text-center italic", "{caption}" }
                }
            }
        }
    } else {
        rsx! {}
    }
}

fn render_linktool_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::LinkTool { data, .. } = block {
        let title = data.meta.title.clone();
        let description = data.meta.description.clone();
        let image_url = data.meta.image.as_ref().map(|img| img.url.clone());
        let url = data.meta.url.clone();

        rsx! {
            a {
                href: "{url}",
                target: "_blank",
                rel: "noopener noreferrer",
                class: "my-4 block rounded-lg border p-4 hover:bg-muted/50 transition-colors",
                div { class: "flex gap-4",
                    if let Some(image_url) = image_url {
                        img {
                            src: "{image_url}",
                            alt: title.as_deref().unwrap_or(""),
                            class: "w-20 h-20 object-cover rounded"
                        }
                    }
                    div { class: "flex-1",
                        if let Some(title) = title {
                            h3 { class: "font-semibold mb-1", "{title}" }
                        }
                        if let Some(description) = description {
                            p { class: "text-sm text-muted-foreground", "{description}" }
                        }
                        p { class: "text-xs text-muted-foreground mt-1", "{url}" }
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}

fn render_attaches_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::Attaches { data, .. } = block {
        let name = data.file.name.clone();
        let size = data.file.size;
        let url = data.file.url.clone();
        let size_mb = size as f64 / 1024.0 / 1024.0;
        let size_str = if size_mb > 1.0 {
            format!("{:.2} MB", size_mb)
        } else {
            format!("{:.2} KB", size as f64 / 1024.0)
        };

        rsx! {
            div { class: "my-4 p-4 rounded-lg border bg-muted/50",
                a {
                    href: "{url}",
                    target: "_blank",
                    class: "flex items-center gap-3 text-sm",
                    "📎 {name} ({size_str})"
                }
            }
        }
    } else {
        rsx! {}
    }
}

fn render_raw_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::Raw { data, .. } = block {
        let html = data.html.clone();
        rsx! {
            div { class: "my-4", dangerous_inner_html: "{html}" }
        }
    } else {
        rsx! {}
    }
}

fn render_table_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::Table { data, .. } = block {
        rsx! {
            div { class: "my-6 overflow-x-auto",
                table { class: "min-w-full border-collapse",
                    tbody {
                        for row in &data.content {
                            tr {
                                for cell in row {
                                    td { class: "border border-muted px-4 py-2", "{cell}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        rsx! {}
    }
}

fn render_warning_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::Warning { data, .. } = block {
        let title = data.title.clone();
        let message = data.message.clone();

        rsx! {
            div { class: "my-6 p-4 rounded-lg border bg-yellow-50 border-yellow-200 text-yellow-800 dark:bg-yellow-950/20 dark:border-yellow-800 dark:text-yellow-300",
                h3 { class: "font-semibold mb-2", "{title}" }
                p { "{message}" }
            }
        }
    } else {
        rsx! {}
    }
}

fn render_button_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::Button { data, .. } = block {
        let text = data.text.clone();
        let link = data.link.clone();
        let style = data.style.as_deref().unwrap_or("primary");

        let button_class = match style {
            "secondary" => "bg-secondary text-secondary-foreground hover:bg-secondary/80",
            "outline" => {
                "border border-input bg-background hover:bg-accent hover:text-accent-foreground"
            }
            "ghost" => "hover:bg-accent hover:text-accent-foreground",
            _ => "bg-primary text-primary-foreground hover:bg-primary/90", // primary
        };

        if let Some(link) = link {
            rsx! {
                a {
                    href: "{link}",
                    target: "_blank",
                    class: "inline-flex items-center justify-center rounded-md text-sm font-medium h-10 px-4 py-2 {button_class}",
                    "{text}"
                }
            }
        } else {
            rsx! {
                button { class: "inline-flex items-center justify-center rounded-md text-sm font-medium h-10 px-4 py-2 {button_class}", "{text}" }
            }
        }
    } else {
        rsx! {}
    }
}

fn render_unknown_block(block: &EditorJsBlock) -> Element {
    if let EditorJsBlock::Unknown { .. } = block {
        rsx! {
            div { class: "my-4 p-3 rounded bg-muted text-sm",
                "Unsupported block type"
            }
        }
    } else {
        rsx! {}
    }
}

fn render_editorjs_content(content: &PostContent) -> Element {
    rsx! {
        div { class: "prose prose-neutral dark:prose-invert max-w-none",
            for block in &content.blocks {
                match block {
                    EditorJsBlock::Header { .. } => render_header_block(block),
                    EditorJsBlock::Paragraph { .. } => render_paragraph_block(block),
                    EditorJsBlock::List { .. } => render_list_block(block),
                    EditorJsBlock::Delimiter { .. } => render_delimiter_block(block),
                    EditorJsBlock::Image { .. } => render_image_block(block),
                    EditorJsBlock::Embed { .. } => render_embed_block(block),
                    EditorJsBlock::LinkTool { .. } => render_linktool_block(block),
                    EditorJsBlock::Attaches { .. } => render_attaches_block(block),
                    EditorJsBlock::Code { .. } => render_code_block(block),
                    EditorJsBlock::Raw { .. } => render_raw_block(block),
                    EditorJsBlock::Table { .. } => render_table_block(block),
                    EditorJsBlock::Quote { .. } => render_quote_block(block),
                    EditorJsBlock::Warning { .. } => render_warning_block(block),
                    EditorJsBlock::Button { .. } => render_button_block(block),
                    EditorJsBlock::Alert { .. } => render_alert_block(block),
                    EditorJsBlock::Checklist { .. } => render_checklist_block(block),
                    EditorJsBlock::Unknown { .. } => render_unknown_block(block),
                }
            }
        }
    }
}

// ============================================================================
// Post View Screen
// ============================================================================

#[component]
pub fn PostsViewScreen(id: i32) -> Element {
    let posts = use_post();
    let nav = use_navigator();

    // Get post by id
    let post = use_memo(move || {
        let posts_read = posts.list.read();
        if let Some(list) = &posts_read.data {
            list.data.iter().find(|p| p.id == id).cloned()
        } else {
            None
        }
    });

    use_effect(move || {
        if post().is_none() {
            let posts_state = posts;
            spawn(async move {
                posts_state.list().await;
            });
        }
    });

    if let Some(post) = post() {
        let published_date = post
            .published_at
            .map(|dt| dt.format("%B %d, %Y").to_string())
            .unwrap_or_else(|| post.created_at.format("%B %d, %Y").to_string());

        rsx! {
            div { class: "min-h-screen bg-transparent text-foreground",
                PageHeader {
                    title: post.title.clone(),
                    description: post.excerpt.clone().unwrap_or_default(),
                    actions: Some(rsx! {
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| { nav.push(Route::PostsListScreen {}); },
                            "Back to Posts"
                        }
                    }),
                }

                div { class: "container mx-auto px-4 py-10 md:py-12",
                    // Featured image
                    if let Some(image) = &post.featured_image {
                        div { class: "mb-8 overflow-hidden rounded-lg",
                            img {
                                src: "{image.file_url}",
                                alt: "{post.title}",
                                class: "w-full h-auto object-cover"
                            }
                        }
                    }

                    // Post meta
                    div { class: "mb-8 pb-4 border-b",
                        div { class: "flex flex-wrap items-center gap-4 text-sm text-muted-foreground",
                            span { "By {post.author.name}" }
                            span { "•" }
                            span { "Published on {published_date}" }
                            if !post.tags.is_empty() {
                                span { "•" }
                                div { class: "flex flex-wrap gap-2",
                                    for tag in &post.tags {
                                        span { class: "px-2 py-1 rounded-full text-xs bg-muted", "{tag.name}" }
                                    }
                                }
                            }
                        }
                    }

                    // Post content
                    div { class: "prose prose-neutral dark:prose-invert max-w-none",
                        {render_editorjs_content(&post.content)}
                    }

                    // Post stats
                    div { class: "mt-10 pt-6 border-t text-sm text-muted-foreground",
                        div { class: "flex flex-wrap gap-6",
                            span { "{post.view_count} views" }
                            span { "{post.likes_count} likes" }
                            span { "{post.comment_count} comments" }
                            span { "Status: {post.status}" }
                        }
                    }
                }
            }
        }
    } else {
        rsx! {
            div { class: "min-h-screen bg-transparent text-foreground",
                PageHeader {
                    title: "Loading...".to_string(),
                    description: "Fetching post content.".to_string(),
                    actions: Some(rsx! {
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |_| { nav.push(Route::PostsListScreen {}); },
                            "Back to Posts"
                        }
                    }),
                }
            }
        }
    }
}
