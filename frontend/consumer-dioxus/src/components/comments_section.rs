use dioxus::prelude::*;
use ruxlog_shared::store::comments::{Comment, CommentCreatePayload};
use ruxlog_shared::store::use_comments;
use ruxlog_shared::store::use_auth;
use hmziq_dioxus_free_icons::icons::ld_icons::{LdMessageCircle, LdSend, LdFlag, LdCornerDownRight, LdLoader};
use hmziq_dioxus_free_icons::Icon;
use oxui::components::error::{ErrorDetails, ErrorDetailsVariant};

#[derive(Props, Clone, PartialEq)]
pub struct CommentsSectionProps {
    pub post_id: i32,
}

/// Comments section for a post
#[component]
pub fn CommentsSection(props: CommentsSectionProps) -> Element {
    let comments_store = use_comments();
    let auth_store = use_auth();
    let post_id = props.post_id;
    
    let mut new_comment = use_signal(String::new);
    let mut reply_to = use_signal(|| None::<i32>);

    // Fetch comments on mount
    use_effect(move || {
        let comments = comments_store;
        spawn(async move {
            comments.list(post_id).await;
        });
    });

    let comments_frame = comments_store.list.read();
    let add_frame = comments_store.add.read();
    let current_user = auth_store.user.read();
    let is_logged_in = current_user.is_some();

    let handle_submit = move |_| {
        let content = new_comment().trim().to_string();
        if content.is_empty() {
            return;
        }

        let payload = CommentCreatePayload {
            post_id,
            content,
            parent_id: reply_to(),
        };

        let comments = comments_store;
        spawn(async move {
            comments.create(payload).await;
        });

        new_comment.set(String::new());
        reply_to.set(None);
    };

    rsx! {
        section { class: "mt-12 pt-8 border-t border-border",
            // Header
            div { class: "flex items-center gap-3 mb-6",
                Icon { icon: LdMessageCircle, class: "w-6 h-6 text-primary" }
                h2 { class: "text-2xl font-bold", "Comments" }
                if let Some(data) = &(*comments_frame).data {
                    span { class: "text-muted-foreground", "({data.data.len()})" }
                }
            }

            // Comment form
            if is_logged_in {
                div { class: "mb-8",
                    if reply_to().is_some() {
                        div { class: "flex items-center gap-2 mb-2 text-sm text-muted-foreground",
                            Icon { icon: LdCornerDownRight, class: "w-4 h-4" }
                            span { "Replying to comment" }
                            button {
                                class: "text-primary hover:underline",
                                onclick: move |_| reply_to.set(None),
                                "Cancel"
                            }
                        }
                    }

                    div { class: "flex gap-3",
                        // Avatar
                        div { class: "flex-shrink-0",
                            div { class: "w-10 h-10 rounded-full bg-primary/10 flex items-center justify-center text-primary font-semibold",
                                if let Some(user) = &*current_user {
                                    "{user.name.chars().next().unwrap_or('U').to_uppercase()}"
                                } else {
                                    "U"
                                }
                            }
                        }

                        // Input
                        div { class: "flex-1",
                            textarea {
                                class: "w-full px-4 py-3 rounded-lg border border-border bg-card/50 text-foreground placeholder-muted-foreground focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary resize-none min-h-[100px]",
                                placeholder: "Share your thoughts...",
                                value: "{new_comment}",
                                oninput: move |e| new_comment.set(e.value()),
                            }

                            // Error display
                            if (*add_frame).is_failed() {
                                div { class: "mt-2",
                                    ErrorDetails {
                                        error: (*add_frame).error.clone(),
                                        variant: ErrorDetailsVariant::Minimum,
                                    }
                                }
                            }

                            div { class: "flex justify-end mt-3",
                                button {
                                    class: "flex items-center gap-2 px-4 py-2 rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed",
                                    disabled: new_comment().trim().is_empty() || (*add_frame).is_loading(),
                                    onclick: handle_submit,
                                    if (*add_frame).is_loading() {
                                        Icon { icon: LdLoader, class: "w-4 h-4 animate-spin" }
                                        "Posting..."
                                    } else {
                                        Icon { icon: LdSend, class: "w-4 h-4" }
                                        "Post comment"
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "mb-8 p-4 rounded-lg border border-border bg-muted/30 text-center",
                    p { class: "text-muted-foreground",
                        "Please "
                        a { class: "text-primary hover:underline", href: "/auth/login", "sign in" }
                        " to leave a comment."
                    }
                }
            }

            // Comments list
            if (*comments_frame).is_loading() {
                CommentsLoadingSkeleton {}
            } else if (*comments_frame).is_failed() {
                div { class: "p-4",
                    ErrorDetails {
                        error: (*comments_frame).error.clone(),
                        variant: ErrorDetailsVariant::Collapsed,
                    }
                }
            } else if let Some(data) = &(*comments_frame).data {
                if data.data.is_empty() {
                    div { class: "text-center py-12",
                        div { class: "w-16 h-16 rounded-full bg-muted/50 flex items-center justify-center mx-auto mb-4",
                            Icon { icon: LdMessageCircle, class: "w-8 h-8 text-muted-foreground" }
                        }
                        p { class: "text-muted-foreground", "No comments yet. Be the first to share your thoughts!" }
                    }
                } else {
                    div { class: "space-y-6",
                        for comment in data.data.iter() {
                            CommentItem {
                                key: "{comment.id}",
                                comment: comment.clone(),
                                on_reply: move |id| reply_to.set(Some(id)),
                                is_logged_in: is_logged_in,
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct CommentItemProps {
    comment: Comment,
    on_reply: EventHandler<i32>,
    is_logged_in: bool,
}

#[component]
fn CommentItem(props: CommentItemProps) -> Element {
    let comment = &props.comment;
    let comment_id = comment.id;
    let created_at = comment.created_at.format("%b %d, %Y at %H:%M").to_string();
    let author_name = comment.author_name();
    let author_initial = author_name.chars().next().unwrap_or('A').to_uppercase();

    rsx! {
        div { class: "flex gap-3",
            // Avatar
            div { class: "flex-shrink-0",
                div { class: "w-10 h-10 rounded-full bg-muted flex items-center justify-center text-muted-foreground font-semibold",
                    "{author_initial}"
                }
            }

            // Content
            div { class: "flex-1 min-w-0",
                div { class: "flex items-center gap-2 mb-1",
                    span { class: "font-semibold text-foreground", "{author_name}" }
                    span { class: "text-xs text-muted-foreground", "{created_at}" }
                }

                p { class: "text-foreground leading-relaxed whitespace-pre-wrap break-words",
                    "{comment.content}"
                }

                // Actions
                div { class: "flex items-center gap-4 mt-2",
                    if props.is_logged_in {
                        button {
                            class: "flex items-center gap-1.5 text-xs text-muted-foreground hover:text-primary transition-colors",
                            onclick: move |_| props.on_reply.call(comment_id),
                            Icon { icon: LdCornerDownRight, class: "w-3.5 h-3.5" }
                            "Reply"
                        }

                        button {
                            class: "flex items-center gap-1.5 text-xs text-muted-foreground hover:text-destructive transition-colors",
                            Icon { icon: LdFlag, class: "w-3.5 h-3.5" }
                            "Report"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CommentsLoadingSkeleton() -> Element {
    rsx! {
        div { class: "space-y-6",
            for _ in 0..3 {
                div { class: "flex gap-3 animate-pulse",
                    div { class: "w-10 h-10 rounded-full bg-muted/50" }
                    div { class: "flex-1 space-y-2",
                        div { class: "flex gap-2",
                            div { class: "h-4 w-24 bg-muted/50 rounded" }
                            div { class: "h-4 w-16 bg-muted/40 rounded" }
                        }
                        div { class: "h-4 w-full bg-muted/40 rounded" }
                        div { class: "h-4 w-3/4 bg-muted/40 rounded" }
                    }
                }
            }
        }
    }
}
