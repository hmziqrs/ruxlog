use dioxus::prelude::*;
use ruxlog_shared::store::{use_newsletter, NewsletterSubscriber, SubscriberListQuery};

#[component]
pub fn NewsletterSubscribersScreen() -> Element {
    let newsletter = use_newsletter();
    let mut search = use_signal(|| "".to_string());
    let mut confirmed_only = use_signal(|| false);

    use_effect(move || {
        spawn(async move {
            newsletter
                .list_subscribers(SubscriberListQuery {
                    confirmed: None,
                    page: 1,
                    limit: Some(50),
                    search: None,
                    ..Default::default()
                })
                .await;
        });
    });

    let reload = move |_| {
        let newsletter = newsletter;
        let q = SubscriberListQuery {
            confirmed: if confirmed_only() { Some(true) } else { None },
            page: 1,
            limit: Some(50),
            search: if search().is_empty() {
                None
            } else {
                Some(search())
            },
            ..Default::default()
        };
        spawn(async move {
            newsletter.list_subscribers(q).await;
        });
    };

    let list = newsletter.subscribers.read();
    let subs: Vec<NewsletterSubscriber> = list
        .data
        .as_ref()
        .map(|p| p.data.clone())
        .unwrap_or_default();

    rsx! {
        div { class: "p-6 space-y-4",
            div { class: "flex items-center justify-between",
                div {
                    h2 { class: "text-2xl font-semibold", "Subscribers" }
                    p { class: "text-sm text-muted-foreground", "Manage newsletter subscribers." }
                }
                button {
                    class: "inline-flex items-center gap-2 rounded-md border border-border px-3 py-2 text-sm hover:bg-accent",
                    onclick: reload,
                    "Refresh"
                }
            }

            div { class: "flex flex-wrap items-center gap-3",
                div { class: "relative flex-1 min-w-[240px]",
                    input {
                        class: "w-full rounded-md border border-border pl-8 pr-3 py-2 text-sm",
                        placeholder: "Search email",
                        value: "{search}",
                        oninput: move |e| search.set(e.value()),
                    }
                }
                label { class: "flex items-center gap-2 text-sm",
                    input { r#type: "checkbox", checked: "{confirmed_only()}", onchange: move |_| confirmed_only.toggle() }
                    "Confirmed only"
                }
                button {
                    class: "rounded-md border border-border px-3 py-2 text-sm font-medium hover:bg-accent",
                    onclick: reload,
                    "Apply"
                }
            }

            div { class: "overflow-auto bg-transparent border border-border rounded-lg",
                table { class: "w-full text-sm",
                    thead { class: "bg-transparent",
                        tr {
                            th { class: "p-3 text-left", "ID" }
                            th { class: "p-3 text-left", "Email" }
                            th { class: "p-3 text-left", "Status" }
                            th { class: "p-3 text-left", "Subscribed" }
                        }
                    }
                    tbody {
                        if subs.is_empty() {
                            tr {
                                td { class: "p-4 text-center text-muted-foreground", colspan: "4",
                                    "No subscribers yet."
                                }
                            }
                        } else {
                            for sub in subs {
                                tr { class: "border-t border-border",
                                    td { class: "p-3", "{sub.id}" }
                                    td { class: "p-3", "{sub.email}" }
                                    td { class: "p-3", if sub.confirmed { "Confirmed" } else { "Pending" } }
                                    td { class: "p-3", "{sub.created_at}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
