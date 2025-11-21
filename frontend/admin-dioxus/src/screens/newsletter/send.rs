use dioxus::prelude::*;
use ruxlog_shared::store::{use_newsletter, SendNewsletterPayload};

use crate::containers::page_header::PageHeader;
use oxui::components::form::input::SimpleInput;
use oxui::shadcn::button::Button;

#[component]
pub fn NewsletterSendScreen() -> Element {
    let newsletter = use_newsletter();
    let mut subject = use_signal(|| "".to_string());
    let mut content = use_signal(|| "".to_string());
    let mut html_content = use_signal(|| "".to_string());

    let send = move |event: FormEvent| {
        event.prevent_default();
        let payload = SendNewsletterPayload {
            subject: subject(),
            content: content(),
            html_content: if html_content().is_empty() {
                None
            } else {
                Some(html_content())
            },
        };
        let newsletter = newsletter;
        spawn(async move {
            newsletter.send(payload).await;
        });
    };

    let send_state = newsletter.send_status.read();
    let last_result = send_state.data.clone().unwrap_or(None);

    rsx! {
        div { class: "min-h-screen bg-transparent",
            PageHeader {
                title: "Send Newsletter".to_string(),
                description: "Send a broadcast to all subscribers".to_string(),
                actions: None,
                class: None,
                embedded: false,
            }

            div { class: "container mx-auto px-4 py-8 md:py-12",
                form { class: "space-y-4 max-w-3xl", onsubmit: send,
                    div { class: "space-y-2",
                        label { class: "text-sm font-medium", "Subject" }
                        SimpleInput {
                            value: subject(),
                            oninput: move |value| subject.set(value),
                            class: Some("text-sm".to_string()),
                        }
                    }
                    div { class: "space-y-2",
                        label { class: "text-sm font-medium", "Content (text)" }
                        textarea {
                            class: "w-full rounded-md border border-border px-3 py-2 text-sm min-h-[160px]",
                            value: "{content}",
                            oninput: move |e| content.set(e.value()),
                            required: true
                        }
                    }
                    div { class: "space-y-2",
                        label { class: "text-sm font-medium", "HTML Content (optional)" }
                        textarea {
                            class: "w-full rounded-md border border-border px-3 py-2 text-sm min-h-[160px]",
                            value: "{html_content}",
                            oninput: move |e| html_content.set(e.value())
                        }
                    }
                    Button {
                        r#type: "submit",
                        "Send"
                    }
                }

                if let Some(result) = last_result {
                    div { class: "rounded-md border border-border px-3 py-2 text-sm mt-4 max-w-3xl",
                        if result.success {
                            span { class: "text-green-600", "{result.message.clone().unwrap_or_else(|| \"Newsletter sent successfully\".to_string())}" }
                        } else {
                            span { class: "text-red-600", "{result.message.clone().unwrap_or_else(|| \"Send failed\".to_string())}" }
                        }
                    }
                }
            }
        }
    }
}
