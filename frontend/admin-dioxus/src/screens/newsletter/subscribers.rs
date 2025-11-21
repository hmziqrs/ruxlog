use dioxus::prelude::*;
use ruxlog_shared::store::{use_newsletter, NewsletterSubscriber, SubscriberListQuery};

use crate::components::table::data_table_screen::{DataTableScreen, HeaderColumn};
use crate::containers::page_header::PageHeaderProps;
use oxui::components::form::input::SimpleInput;
use oxui::shadcn::button::{Button, ButtonVariant};

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

    // Define header columns
    let headers = vec![
        HeaderColumn::new("ID", false, "p-3 text-left font-medium text-xs md:text-sm", None),
        HeaderColumn::new("Email", false, "p-3 text-left font-medium text-xs md:text-sm", None),
        HeaderColumn::new("Status", false, "p-3 text-left font-medium text-xs md:text-sm", None),
        HeaderColumn::new("Subscribed", false, "p-3 text-left font-medium text-xs md:text-sm", None),
    ];

    rsx! {
        DataTableScreen::<NewsletterSubscriber> {
            frame: (newsletter.subscribers)(),
            header: Some(PageHeaderProps {
                title: "Newsletter Subscribers".to_string(),
                description: "Manage newsletter subscribers".to_string(),
                actions: Some(rsx!{
                    Button {
                        onclick: reload,
                        "Refresh"
                    }
                }),
                class: None,
                embedded: false,
            }),
            headers: Some(headers),
            current_sort_field: None,
            on_sort: None,
            show_pagination: false,
            on_prev: move |_| {},
            on_next: move |_| {},
            below_toolbar: Some(rsx! {
                div { class: "flex flex-wrap items-center gap-3",
                    div { class: "relative flex-1 min-w-[240px]",
                        SimpleInput {
                            placeholder: Some("Search email".to_string()),
                            value: search(),
                            oninput: move |value| search.set(value),
                            class: Some("text-sm".to_string()),
                        }
                    }
                    label { class: "flex items-center gap-2 text-sm",
                        input { r#type: "checkbox", checked: "{confirmed_only()}", onchange: move |_| confirmed_only.toggle() }
                        "Confirmed only"
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: reload,
                        "Apply"
                    }
                }
            }),
            if subs.is_empty() {
                tr {
                    td { class: "p-4 text-center text-muted-foreground", colspan: "4",
                        "No subscribers yet."
                    }
                }
            } else {
                {subs.iter().cloned().map(|sub| {
                    rsx! {
                        tr { key: "{sub.id}", class: "border-b border-zinc-200 dark:border-zinc-800 hover:bg-muted/30 transition-colors",
                            td { class: "p-3", "{sub.id}" }
                            td { class: "p-3", "{sub.email}" }
                            td { class: "p-3", if sub.confirmed { "Confirmed" } else { "Pending" } }
                            td { class: "p-3", "{sub.created_at}" }
                        }
                    }
                })}
            }
        }
    }
}
