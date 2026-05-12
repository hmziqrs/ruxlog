use crate::seo::{breadcrumb_schema, use_static_seo, SeoHead, StructuredData};
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::icons::ld_icons::{LdClock, LdMail, LdMapPin};
use hmziq_dioxus_free_icons::Icon;
use oxui::shadcn::button::{Button, ButtonVariant};

#[component]
pub fn ContactScreen() -> Element {
    let seo_metadata = use_static_seo("contact");

    let mut name = use_signal(String::new);
    let mut email = use_signal(String::new);
    let mut message = use_signal(String::new);
    let mut submitted = use_signal(|| false);
    let mut sending = use_signal(|| false);

    let on_submit = move |_| {
        let name_val = name.read().clone();
        let email_val = email.read().clone();
        let message_val = message.read().clone();

        if name_val.trim().is_empty()
            || email_val.trim().is_empty()
            || message_val.trim().is_empty()
        {
            return;
        }

        spawn(async move {
            sending.set(true);
            // TODO: Wire to POST /contact/v1/submit when endpoint is added
            sending.set(false);
            submitted.set(true);
        });
    };

    rsx! {
        SeoHead { metadata: seo_metadata }
        StructuredData {
            json_ld: breadcrumb_schema(vec![
                ("Home", "/"),
                ("Contact", "/contact"),
            ])
        }

        div { class: "min-h-screen",
            div { class: "container mx-auto px-4 py-8 md:py-12 lg:py-16 max-w-4xl",
                // Header
                div { class: "text-center mb-12",
                    h1 { class: "text-4xl md:text-5xl font-bold mb-4", "Get in Touch" }
                    p { class: "text-lg text-muted-foreground max-w-xl mx-auto",
                        "Have a question, suggestion, or just want to say hello? We'd love to hear from you."
                    }
                }

                div { class: "grid md:grid-cols-3 gap-8",
                    // Contact info sidebar
                    div { class: "space-y-6",
                        ContactInfo {
                            icon: rsx! { Icon { icon: LdMail, class: "w-5 h-5" } },
                            title: "Email",
                            detail: "hello@hmziq.rs",
                        }
                        ContactInfo {
                            icon: rsx! { Icon { icon: LdMapPin, class: "w-5 h-5" } },
                            title: "Location",
                            detail: "Remote-first",
                        }
                        ContactInfo {
                            icon: rsx! { Icon { icon: LdClock, class: "w-5 h-5" } },
                            title: "Response Time",
                            detail: "Within 48 hours",
                        }
                    }

                    // Contact form
                    div { class: "md:col-span-2",
                        if submitted() {
                            div { class: "rounded-xl border border-border bg-card p-8 text-center",
                                h2 { class: "text-2xl font-bold mb-2", "Message Sent!" }
                                p { class: "text-muted-foreground mb-4",
                                    "Thanks for reaching out. We'll get back to you soon."
                                }
                                Button {
                                    variant: ButtonVariant::Outline,
                                    onclick: move |_| submitted.set(false),
                                    "Send another message"
                                }
                            }
                        } else {
                            form {
                                class: "space-y-6",
                                onsubmit: on_submit,
                                div { class: "space-y-2",
                                    label { class: "text-sm font-medium", "Name" }
                                    input {
                                        r#type: "text",
                                        class: "w-full rounded-lg border border-border bg-background px-4 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-primary/50",
                                        placeholder: "Your name",
                                        value: "{name}",
                                        oninput: move |e| name.set(e.value()),
                                        required: true,
                                    }
                                }
                                div { class: "space-y-2",
                                    label { class: "text-sm font-medium", "Email" }
                                    input {
                                        r#type: "email",
                                        class: "w-full rounded-lg border border-border bg-background px-4 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-primary/50",
                                        placeholder: "you@example.com",
                                        value: "{email}",
                                        oninput: move |e| email.set(e.value()),
                                        required: true,
                                    }
                                }
                                div { class: "space-y-2",
                                    label { class: "text-sm font-medium", "Message" }
                                    textarea {
                                        class: "w-full rounded-lg border border-border bg-background px-4 py-2.5 text-sm focus:outline-none focus:ring-2 focus:ring-primary/50 min-h-[150px] resize-y",
                                        placeholder: "What's on your mind?",
                                        value: "{message}",
                                        oninput: move |e| message.set(e.value()),
                                        required: true,
                                    }
                                }
                                Button {
                                    variant: ButtonVariant::Default,
                                    r#type: "submit",
                                    disabled: sending(),
                                    class: "w-full",
                                    if sending() {
                                        "Sending..."
                                    } else {
                                        "Send Message"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ContactInfo(icon: Element, title: &'static str, detail: &'static str) -> Element {
    rsx! {
        div { class: "flex items-start gap-3",
            div { class: "text-primary mt-0.5", {icon} }
            div {
                h3 { class: "font-medium text-sm", "{title}" }
                p { class: "text-sm text-muted-foreground", "{detail}" }
            }
        }
    }
}
