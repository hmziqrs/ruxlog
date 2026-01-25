use dioxus::prelude::*;
use hmziq_dioxus_free_icons::icons::ld_icons::{LdHeart, LdLoader};
use hmziq_dioxus_free_icons::Icon;

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
    pub on_share: Option<EventHandler<()>>,
    #[props(into)]
    pub on_scroll_to_comments: Option<EventHandler<()>>,
}

/// Engagement bar with views, likes, comments, and share
#[component]
pub fn EngagementBar(props: EngagementBarProps) -> Element {
    use hmziq_dioxus_free_icons::icons::ld_icons::{LdEye, LdMessageCircle, LdShare2};

    rsx! {
        div { class: "flex items-center justify-between",
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

        // Share button
        button {
            class: "share-button",
            onclick: move |_| {
                if let Some(handler) = &props.on_share {
                    handler.call(());
                    }
                },
                Icon { icon: LdShare2, class: "w-4 h-4" }
                span { class: "font-semibold text-sm", "Share" }
            }
        }
    }
}
