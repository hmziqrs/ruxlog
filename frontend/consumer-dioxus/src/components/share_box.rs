use dioxus::prelude::*;
use hmziq_dioxus_free_icons::icons::ld_icons::{
    LdX, LdFacebook, LdLinkedin, LdMessageCircle, LdMail, LdSearch, LdChevronDown, LdChevronUp,
    LdCopy, LdCheck,
};
use hmziq_dioxus_free_icons::Icon;

#[cfg(feature = "analytics")]
use crate::analytics::tracker;

#[component]
pub fn ShareBox(
    show: Signal<bool>,
    post_id: String,
    title: String,
    url: String,
) -> Element {
    let mut search_query = use_signal(|| String::new());
    let mut show_more = use_signal(|| false);
    let mut copy_success = use_signal(|| false);

    // Define all platforms
    let platforms = vec![
        // Primary platforms (always visible)
        Platform {
            name: "X (Twitter)",
            icon_name: "x",
            url_template: "https://twitter.com/intent/tweet?text={title}&url={url}",
            is_primary: true,
        },
        Platform {
            name: "Reddit",
            icon_name: "reddit",
            url_template: "https://reddit.com/submit?title={title}&url={url}",
            is_primary: true,
        },
        Platform {
            name: "Facebook",
            icon_name: "facebook",
            url_template: "https://www.facebook.com/sharer/sharer.php?u={url}",
            is_primary: true,
        },
        Platform {
            name: "Telegram",
            icon_name: "telegram",
            url_template: "https://t.me/share/url?url={url}&text={title}",
            is_primary: true,
        },
        Platform {
            name: "WhatsApp",
            icon_name: "whatsapp",
            url_template: "https://wa.me/?text={title}%20{url}",
            is_primary: true,
        },
        Platform {
            name: "LinkedIn",
            icon_name: "linkedin",
            url_template: "https://www.linkedin.com/sharing/share-offsite/?url={url}",
            is_primary: true,
        },
        // Secondary platforms (show more)
        Platform {
            name: "Discord",
            icon_name: "discord",
            url_template: "https://discord.com/channels/@me",
            is_primary: false,
        },
        Platform {
            name: "Pinterest",
            icon_name: "pinterest",
            url_template: "https://pinterest.com/pin/create/button/?url={url}&description={title}",
            is_primary: false,
        },
        Platform {
            name: "Tumblr",
            icon_name: "tumblr",
            url_template: "https://www.tumblr.com/widgets/share/tool?canonicalUrl={url}&title={title}",
            is_primary: false,
        },
        Platform {
            name: "Mastodon",
            icon_name: "mastodon",
            url_template: "https://mastodonshare.com/?text={title}&url={url}",
            is_primary: false,
        },
        Platform {
            name: "Bluesky",
            icon_name: "bluesky",
            url_template: "https://bsky.app/intent/compose?text={title}%20{url}",
            is_primary: false,
        },
        Platform {
            name: "Threads",
            icon_name: "threads",
            url_template: "https://www.threads.net/intent/post?text={title}%20{url}",
            is_primary: false,
        },
        Platform {
            name: "Email",
            icon_name: "email",
            url_template: "mailto:?subject={title}&body={url}",
            is_primary: false,
        },
    ];

    // Filter platforms based on search query
    let filtered_platforms = use_memo(move || {
        let query = search_query.read().to_lowercase();
        if query.is_empty() {
            platforms.clone()
        } else {
            platforms
                .iter()
                .filter(|p| p.name.to_lowercase().contains(&query))
                .cloned()
                .collect()
        }
    });

    // Determine which platforms to show
    let visible_platforms = use_memo(move || {
        let filtered = filtered_platforms();
        let query = search_query.read();

        if !query.is_empty() {
            // When searching, show all filtered results
            filtered
        } else if show_more() {
            // Show all platforms
            filtered
        } else {
            // Show only primary platforms
            filtered.into_iter().filter(|p| p.is_primary).collect()
        }
    });

    // Check if we have results
    let has_results = use_memo(move || !visible_platforms().is_empty());

    // Handle share click
    let handle_share = move |platform_name: &str, share_url: &str| {
        let platform_name = platform_name.to_string();
        let share_url = share_url.to_string();

        // Track share event
        #[cfg(feature = "analytics")]
        {
            tracker::track_share(&post_id, &title, &platform_name);
        }

        // Open share URL
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let _ = window.open_with_url_and_target(&share_url, "_blank");
            }
        }
    };

    // Handle copy link
    let handle_copy_link = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(navigator) = window.navigator().clipboard() {
                    let promise = navigator.write_text(&url);
                    wasm_bindgen_futures::spawn_local(async move {
                        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                        copy_success.set(true);

                        // Track copy link
                        #[cfg(feature = "analytics")]
                        {
                            tracker::track_share(&post_id, &title, "copy_link");
                        }

                        // Reset success message after 2 seconds
                        gloo_timers::future::TimeoutFuture::new(2000).await;
                        copy_success.set(false);
                    });
                }
            }
        }
    };

    // Handle clear search
    let handle_clear_search = move |_| {
        search_query.set(String::new());
    };

    if !show() {
        return rsx! {};
    }

    rsx! {
        // Modal backdrop
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50",
            onclick: move |_| show.set(false),

            // Modal content
            div {
                class: "bg-background rounded-lg shadow-xl max-w-md w-full max-h-[90vh] overflow-hidden",
                onclick: move |e| e.stop_propagation(),

                // Header
                div { class: "flex items-center justify-between p-4 border-b border-border",
                    h3 { class: "text-lg font-semibold", "Share this post" }
                    button {
                        class: "p-2 hover:bg-muted rounded-lg transition-colors",
                        onclick: move |_| show.set(false),
                        Icon { icon: LdX, class: "w-5 h-5" }
                    }
                }

                // Search bar
                div { class: "p-4 border-b border-border",
                    div { class: "relative",
                        Icon {
                            icon: LdSearch,
                            class: "absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground",
                        }
                        input {
                            r#type: "text",
                            class: "w-full pl-10 pr-4 py-2 bg-muted rounded-lg border border-border focus:outline-none focus:ring-2 focus:ring-primary",
                            placeholder: "Search platforms...",
                            value: "{search_query}",
                            oninput: move |e| search_query.set(e.value().clone()),
                        }
                    }
                }

                // Content
                div { class: "p-4 overflow-y-auto max-h-[400px]",
                    if has_results() {
                        // Platforms grid
                        div { class: "grid grid-cols-3 gap-3 mb-4",
                            for platform in visible_platforms() {
                                {
                                    let share_url = build_share_url(&platform.url_template, &title, &url);
                                    let platform_name = platform.name.clone();
                                    rsx! {
                                        button {
                                            key: "{platform.name}",
                                            class: "flex flex-col items-center gap-2 p-3 hover:bg-muted rounded-lg transition-colors group",
                                            onclick: move |_| handle_share(&platform_name, &share_url),
                                            div { class: "w-12 h-12 rounded-full bg-primary/10 flex items-center justify-center group-hover:bg-primary/20 transition-colors",
                                                { get_platform_icon(&platform.icon_name) }
                                            }
                                            span { class: "text-xs text-center line-clamp-2", "{platform.name}" }
                                        }
                                    }
                                }
                            }
                        }

                        // Show More/Less button (only when not searching)
                        if search_query.read().is_empty() {
                            div { class: "flex justify-center mb-4",
                                button {
                                    class: "flex items-center gap-2 px-4 py-2 text-sm hover:bg-muted rounded-lg transition-colors",
                                    onclick: move |_| show_more.set(!show_more()),
                                    if show_more() {
                                        Icon { icon: LdChevronUp, class: "w-4 h-4" }
                                        "Show Less"
                                    } else {
                                        Icon { icon: LdChevronDown, class: "w-4 h-4" }
                                        "Show More"
                                    }
                                }
                            }
                        }
                    } else {
                        // No results
                        div { class: "text-center py-8",
                            p { class: "text-muted-foreground mb-4", "No platforms found" }
                            button {
                                class: "px-4 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary/90 transition-colors",
                                onclick: handle_clear_search,
                                "Clear Search"
                            }
                        }
                    }

                    // Copy Link button
                    div { class: "pt-4 border-t border-border",
                        button {
                            class: "w-full flex items-center justify-center gap-2 px-4 py-3 bg-muted hover:bg-muted/80 rounded-lg transition-colors",
                            onclick: handle_copy_link,
                            if copy_success() {
                                Icon { icon: LdCheck, class: "w-5 h-5 text-green-500" }
                                span { class: "text-green-500", "Link Copied!" }
                            } else {
                                Icon { icon: LdCopy, class: "w-5 h-5" }
                                "Copy Link"
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
struct Platform {
    name: &'static str,
    icon_name: &'static str,
    url_template: &'static str,
    is_primary: bool,
}

fn build_share_url(template: &str, title: &str, url: &str) -> String {
    template
        .replace("{title}", &urlencoding::encode(title))
        .replace("{url}", &urlencoding::encode(url))
}

fn get_platform_icon(icon_name: &str) -> Element {
    match icon_name {
        "x" => rsx! {
            Icon { icon: LdX, class: "w-6 h-6" }
        },
        "facebook" => rsx! {
            Icon { icon: LdFacebook, class: "w-6 h-6" }
        },
        "linkedin" => rsx! {
            Icon { icon: LdLinkedin, class: "w-6 h-6" }
        },
        "email" => rsx! {
            Icon { icon: LdMail, class: "w-6 h-6" }
        },
        "reddit" | "telegram" | "whatsapp" | "discord" | "pinterest" | "tumblr" | "mastodon" | "bluesky" | "threads" => rsx! {
            Icon { icon: LdMessageCircle, class: "w-6 h-6" }
        },
        _ => rsx! {
            Icon { icon: LdMessageCircle, class: "w-6 h-6" }
        },
    }
}
