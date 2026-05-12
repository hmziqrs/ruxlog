use crate::config::BRAND;
use crate::router::Route;
use crate::seo::{breadcrumb_schema, SeoHead, SeoMetadataBuilder, StructuredData};
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::icons::ld_icons::{LdArrowLeft, LdSearch};
use hmziq_dioxus_free_icons::Icon;
use oxui::shadcn::button::{Button, ButtonVariant};

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
struct SearchMeta {
    total: u64,
    page: u64,
    per_page: u64,
    query: String,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
struct SearchResponse {
    data: Vec<SearchHit>,
    meta: SearchMeta,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
struct SearchHit {
    id: i32,
    title: String,
    slug: String,
    excerpt: Option<String>,
    published_at: Option<String>,
    created_at: String,
}

#[component]
pub fn SearchScreen() -> Element {
    let nav = use_navigator();
    let mut query = use_signal(String::new);
    let mut results = use_signal(Vec::<SearchHit>::new);
    let mut meta = use_signal(|| Option::<SearchMeta>::None);
    let mut loading = use_signal(|| false);
    let mut searched = use_signal(|| false);
    let mut error_msg = use_signal(|| Option::<String>::None);

    let seo_metadata = SeoMetadataBuilder::new()
        .title("Search")
        .description(&format!("Search posts on {}", BRAND.app_name))
        .canonical("/search")
        .build();

    let on_search = move |_| {
        let q = query.read().clone();
        if q.trim().is_empty() {
            return;
        }
        let q_clone = q.clone();
        spawn(async move {
            loading.set(true);
            error_msg.set(None);
            searched.set(true);

            let response =
                oxcore::http::post("/search/v1/search", &serde_json::json!({ "q": q_clone }))
                    .send()
                    .await;

            match response {
                Ok(resp) => {
                    if (200..300).contains(&resp.status()) {
                        match resp.json::<SearchResponse>().await {
                            Ok(data) => {
                                results.set(data.data);
                                meta.set(Some(data.meta));
                            }
                            Err(e) => {
                                error_msg.set(Some(format!("Failed to parse results: {}", e)));
                            }
                        }
                    } else {
                        error_msg.set(Some(format!("Search failed: {}", resp.status())));
                    }
                }
                Err(e) => {
                    error_msg.set(Some(format!("Network error: {}", e)));
                }
            }
            loading.set(false);
        });
    };

    let on_post_click = move |post_slug: String| {
        nav.push(Route::PostViewScreen { slug: post_slug });
    };

    rsx! {
        SeoHead { metadata: seo_metadata }
        StructuredData {
            json_ld: breadcrumb_schema(vec![
                ("Home", "/"),
                ("Search", "/search"),
            ])
        }

        div { class: "min-h-screen",
            div { class: "container mx-auto px-4 py-8 md:py-12 lg:py-16 max-w-3xl",
                // Search header
                div { class: "text-center mb-8",
                    h1 { class: "text-3xl md:text-4xl font-bold mb-4", "Search Posts" }
                    p { class: "text-muted-foreground mb-8",
                        "Find articles, tutorials, and stories across the blog."
                    }
                    form {
                        class: "flex gap-2 max-w-xl mx-auto",
                        onsubmit: on_search,
                        div { class: "relative flex-1",
                            Icon {
                                icon: LdSearch,
                                class: "absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-muted-foreground"
                            }
                            input {
                                r#type: "text",
                                class: "w-full rounded-lg border border-border bg-background pl-10 pr-4 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-primary/50",
                                placeholder: "Search by title, topic, or keyword...",
                                value: "{query}",
                                oninput: move |e| query.set(e.value()),
                            }
                        }
                        Button {
                            variant: ButtonVariant::Default,
                            r#type: "submit",
                            disabled: loading(),
                            if loading() {
                                "Searching..."
                            } else {
                                "Search"
                            }
                        }
                    }
                }

                // Error state
                if let Some(err) = error_msg() {
                    div { class: "text-center py-8 text-destructive", "{err}" }
                }

                // Results
                if searched() {
                    // Results count
                    if let Some(m) = meta() {
                        {
                            let suffix = if m.total == 1 { "" } else { "s" };
                            let label = format!("{} result{} for \"{}\"", m.total, suffix, m.query);
                            rsx! { div { class: "text-sm text-muted-foreground mb-6", "{label}" } }
                        }
                    }

                    if results().is_empty() && !loading() {
                        div { class: "text-center py-16",
                            h2 { class: "text-xl font-semibold mb-2", "No results found" }
                            p { class: "text-muted-foreground",
                                "Try different keywords or browse posts from the homepage."
                            }
                        }
                    } else {
                        div { class: "space-y-4",
                            for hit in results().iter() {
                                SearchResultCard {
                                    key: "{hit.id}",
                                    hit: hit.clone(),
                                    on_click: on_post_click,
                                }
                            }
                        }
                    }
                } else {
                    // Initial state
                    div { class: "text-center py-16",
                        h2 { class: "text-xl font-semibold mb-2", "Start Searching" }
                        p { class: "text-muted-foreground",
                            "Enter a keyword above to find posts."
                        }
                    }
                }

                div { class: "flex justify-center pt-8",
                    Button {
                        variant: ButtonVariant::Ghost,
                        class: "h-10 px-4 rounded-lg hover:bg-muted/60",
                        onclick: move |_| { nav.push(Route::HomeScreen {}); },
                        Icon { icon: LdArrowLeft, class: "w-4 h-4" }
                        "Back to all posts"
                    }
                }
            }
        }
    }
}

#[component]
fn SearchResultCard(hit: SearchHit, on_click: EventHandler<String>) -> Element {
    let date = hit.published_at.as_deref().unwrap_or(&hit.created_at);
    let excerpt = hit.excerpt.as_deref().unwrap_or("");

    rsx! {
        div {
            class: "rounded-lg border border-border bg-card p-5 hover:shadow-md transition-shadow cursor-pointer",
            onclick: move |_| on_click.call(hit.slug.clone()),
            h3 { class: "text-lg font-semibold mb-1 hover:text-primary transition-colors", "{hit.title}" }
            if !excerpt.is_empty() {
                p { class: "text-sm text-muted-foreground line-clamp-2 mb-2", "{excerpt}" }
            }
            p { class: "text-xs text-muted-foreground", "{date}" }
        }
    }
}
