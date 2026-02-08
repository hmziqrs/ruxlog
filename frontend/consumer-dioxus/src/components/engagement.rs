use dioxus::prelude::*;
use hmziq_dioxus_free_icons::icons::ld_icons::{
    LdBookmarkPlus, LdHeart, LdLink2, LdLoader, LdPrinter, LdShare2,
};
use hmziq_dioxus_free_icons::Icon;

use super::ShareBox;

#[cfg(feature = "analytics")]
use crate::analytics::tracker;

#[derive(Props, Clone, PartialEq)]
pub struct LikeButtonProps {
    pub likes_count: i32,
    #[props(default = false)]
    pub is_liked: bool,
    #[props(default = false)]
    pub is_loading: bool,
    #[props(default = false)]
    pub disabled: bool,
    #[props(into)]
    pub on_click: Option<EventHandler<()>>,
}

/// Like/heart button for posts with loading state
#[component]
pub fn LikeButton(props: LikeButtonProps) -> Element {
    let is_liked = props.is_liked;
    let is_loading = props.is_loading;
    let disabled = props.disabled || is_loading;

    let button_class = if is_liked {
        "like-button-liked"
    } else {
        "like-button"
    };

    let heart_class = if is_liked {
        "w-4 h-4 fill-current"
    } else {
        "w-4 h-4"
    };

    rsx! {
        button {
            class: "{button_class} disabled:opacity-50 disabled:cursor-not-allowed",
            disabled,
            onclick: move |_| {
                if let Some(handler) = &props.on_click {
                    handler.call(());
                }
            },
            if is_loading {
                Icon { icon: LdLoader, class: "w-4 h-4 animate-spin" }
            } else {
                Icon { icon: LdHeart, class: "{heart_class}" }
            }
            span { class: "engagement-count", "{props.likes_count}" }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ShareButtonProps {
    #[props(into)]
    pub on_click: Option<EventHandler<()>>,
}

/// Share button for posts
#[component]
pub fn ShareButton(props: ShareButtonProps) -> Element {
    rsx! {
        button {
            class: "share-button",
            onclick: move |_| {
                if let Some(handler) = &props.on_click {
                    handler.call(());
                }
            },
            Icon { icon: LdShare2, class: "w-4 h-4" }
            span { class: "font-semibold text-sm", "Share" }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct EngagementBarProps {
    pub view_count: i32,
    pub likes_count: i32,
    pub comment_count: i64,
    #[props(default = false)]
    pub is_liked: bool,
    #[props(default = false)]
    pub is_like_loading: bool,
    #[props(into)]
    pub on_like: Option<EventHandler<()>>,
    #[props(into)]
    pub on_scroll_to_comments: Option<EventHandler<()>>,
    #[props(into, default)]
    pub post_id: Option<String>,
    #[props(into, default)]
    pub post_title: Option<String>,
}

/// Engagement bar with views, likes, and comments
#[component]
pub fn EngagementBar(props: EngagementBarProps) -> Element {
    use hmziq_dioxus_free_icons::icons::ld_icons::{LdEye, LdMessageCircle};

    let is_liked = props.is_liked;
    let post_id = props.post_id.clone();
    let post_title = props.post_title.clone();

    rsx! {
        div { class: "flex items-center gap-3",
            // Views (display only)
            div { class: "engagement-button",
                Icon { icon: LdEye, class: "w-4 h-4" }
                span { class: "engagement-count", "{props.view_count}" }
            }

            // Likes button
            LikeButton {
                likes_count: props.likes_count,
                is_liked: props.is_liked,
                is_loading: props.is_like_loading,
                on_click: move |_| {
                    // Track like event
                    #[cfg(feature = "analytics")]
                    if let (Some(pid), Some(title)) = (&post_id, &post_title) {
                        tracker::track_like(pid, title, !is_liked);
                    }

                    if let Some(handler) = &props.on_like {
                        handler.call(());
                    }
                },
            }

            // Comments (clickable to scroll)
            button {
                class: "comment-button",
                onclick: move |_| {
                    if let Some(handler) = &props.on_scroll_to_comments {
                        handler.call(());
                    }
                },
                Icon { icon: LdMessageCircle, class: "w-4 h-4" }
                span { class: "engagement-count", "{props.comment_count}" }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ActionBarProps {
    #[props(into)]
    pub post_id: String,
    #[props(into)]
    pub title: String,
    #[props(into)]
    pub url: String,
}

/// Action bar with utility buttons: Share, Copy Link, Print, Bookmark
#[component]
pub fn ActionBar(props: ActionBarProps) -> Element {
    let mut show_bookmark_hint = use_signal(|| false);
    let mut feedback_message = use_signal(|| String::new());
    let mut show_share_modal = use_signal(|| false);

    // Store props in signals for use in closures
    let post_id = use_signal(|| props.post_id.clone());
    let title = use_signal(|| props.title.clone());
    let url = use_signal(|| props.url.clone());

    // Handle share - opens ShareBox modal
    let handle_share = move |_| {
        show_share_modal.set(true);
    };

    // Handle copy link
    #[allow(unused_variables)]
    let handle_copy_link = move |_| {
        let url = url();
        spawn(async move {
            #[cfg(target_arch = "wasm32")]
            {
                if let Some(window) = web_sys::window() {
                    let clipboard = window.navigator().clipboard();
                    let _ = clipboard.write_text(&url);
                    feedback_message.set("Link copied!".to_string());
                    gloo_timers::future::TimeoutFuture::new(2000).await;
                    feedback_message.set(String::new());
                }
            }
        });
    };

    // Handle print
    let handle_print = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let _ = window.print();
            }
        }
    };

    // Handle bookmark hint
    let handle_bookmark = move |_| {
        show_bookmark_hint.set(true);
        spawn(async move {
            #[cfg(target_arch = "wasm32")]
            {
                gloo_timers::future::TimeoutFuture::new(3000).await;
            }
            show_bookmark_hint.set(false);
        });
    };

    // Detect if user is on Mac
    let is_mac = {
        #[cfg(target_arch = "wasm32")]
        {
            web_sys::window()
                .and_then(|w| w.navigator().user_agent().ok())
                .map(|ua| ua.contains("Mac"))
                .unwrap_or(false)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            false
        }
    };

    let shortcut_key = if is_mac { "⌘D" } else { "Ctrl+D" };

    rsx! {
        div { class: "flex flex-col items-center gap-6 py-6 border-y border-border",
            // Intro text
            div { class: "text-center space-y-1",
                p { class: "text-sm font-medium text-foreground/90",
                    "Found this helpful?"
                }
                p { class: "text-xs text-muted-foreground",
                    "Share it with others or save it for later"
                }
            }

            // Action buttons
            div { class: "flex flex-wrap items-center justify-center gap-3",
                // Share button - Primary action
                button {
                    class: "group flex items-center gap-2 px-5 py-2.5 bg-primary text-primary-foreground rounded-lg font-medium text-sm transition-all hover:bg-primary/90 hover:shadow-md hover:scale-105 active:scale-95",
                    onclick: handle_share,
                    Icon { icon: LdShare2, class: "w-4 h-4 transition-transform group-hover:rotate-12" }
                    span { "Share" }
                }

                // Secondary actions
                div { class: "flex items-center gap-2",
                    // Copy Link button
                    button {
                        class: "group flex items-center gap-2 px-4 py-2.5 bg-muted hover:bg-muted/80 rounded-lg text-sm font-medium transition-all hover:shadow-sm hover:scale-105 active:scale-95",
                        onclick: handle_copy_link,
                        Icon { icon: LdLink2, class: "w-4 h-4 transition-transform group-hover:-rotate-12" }
                        span { "Copy Link" }
                    }

                    // Print button
                    button {
                        class: "group flex items-center gap-2 px-4 py-2.5 bg-muted hover:bg-muted/80 rounded-lg text-sm font-medium transition-all hover:shadow-sm hover:scale-105 active:scale-95",
                        onclick: handle_print,
                        Icon { icon: LdPrinter, class: "w-4 h-4" }
                        span { "Print" }
                    }

                    // Bookmark button
                    div { class: "relative",
                        button {
                            class: "group flex items-center gap-2 px-4 py-2.5 bg-muted hover:bg-muted/80 rounded-lg text-sm font-medium transition-all hover:shadow-sm hover:scale-105 active:scale-95",
                            onclick: handle_bookmark,
                            Icon { icon: LdBookmarkPlus, class: "w-4 h-4 transition-transform group-hover:scale-110" }
                            span { "Bookmark" }
                        }

                        // Bookmark hint tooltip - Modern design
                        if show_bookmark_hint() {
                            div {
                                class: "absolute bottom-full left-1/2 -translate-x-1/2 mb-3 animate-fade-in",
                                div {
                                    class: "px-3 py-2 bg-gray-900 dark:bg-gray-100 text-white dark:text-gray-900 text-xs font-medium rounded-md shadow-lg whitespace-nowrap",
                                    "Press "
                                    kbd { class: "px-1.5 py-0.5 bg-white/20 dark:bg-gray-900/20 rounded text-xs font-mono", "{shortcut_key}" }
                                    " to bookmark"
                                }
                                // Arrow
                                div {
                                    class: "absolute top-full left-1/2 -translate-x-1/2 -mt-1 w-0 h-0 border-l-4 border-r-4 border-t-4 border-transparent border-t-gray-900 dark:border-t-gray-100"
                                }
                            }
                        }
                    }
                }
            }

            // Feedback message - Improved design
            if !feedback_message().is_empty() {
                div {
                    class: "flex items-center gap-2 px-4 py-2 bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-300 rounded-md text-sm font-medium animate-fade-in",
                    svg {
                        class: "w-4 h-4",
                        "xmlns": "http://www.w3.org/2000/svg",
                        "fill": "none",
                        "viewBox": "0 0 24 24",
                        "stroke": "currentColor",
                        path {
                            "stroke-linecap": "round",
                            "stroke-linejoin": "round",
                            "stroke-width": "2",
                            "d": "M5 13l4 4L19 7"
                        }
                    }
                    "{feedback_message()}"
                }
            }
        }

        // ShareBox modal
        ShareBox {
            show: show_share_modal,
            post_id: post_id(),
            title: title(),
            url: url(),
        }
    }
}
