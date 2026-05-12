use dioxus::prelude::*;

use crate::containers::page_header::PageHeader;
use oxui::shadcn::button::{Button, ButtonVariant};
use oxui::shadcn::card::Card;
use oxui::shadcn::checkbox::Checkbox;

#[component]
pub fn NotificationSettingsScreen() -> Element {
    // Toggle states for each email notification event
    let mut new_comment = use_signal(|| true);
    let mut new_user = use_signal(|| true);
    let mut payment_received = use_signal(|| true);
    let mut subscription_cancelled = use_signal(|| true);
    let mut newsletter_subscriber = use_signal(|| false);
    let mut contact_form = use_signal(|| true);

    // Channel toggles
    let mut channel_email = use_signal(|| true);
    let mut channel_in_app = use_signal(|| false);
    let mut channel_webhook = use_signal(|| false);

    // Webhook URL
    let mut webhook_url = use_signal(|| String::new());

    rsx! {
        div { class: "min-h-screen bg-transparent text-foreground",
            PageHeader {
                title: "Notification Settings".to_string(),
                description: "Configure how and when you receive notifications.".to_string(),
            }

            div { class: "container mx-auto px-4 my-8 space-y-8",

                // -- Email Notification Preferences --
                div { class: "space-y-4",
                    div {
                        h2 { class: "text-lg font-semibold", "Email Notifications" }
                        p { class: "text-sm text-muted-foreground",
                            "Choose which events trigger email notifications."
                        }
                    }

                    Card { class: "p-6 divide-y divide-border",
                        // New comment on post
                        div { class: "flex items-center justify-between py-4 first:pt-0",
                            div { class: "space-y-0.5 pr-4",
                                p { class: "text-sm font-medium", "New comment on post" }
                                p { class: "text-xs text-muted-foreground",
                                    "Get notified when someone comments on your blog post."
                                }
                            }
                            Checkbox {
                                checked: *new_comment.read(),
                                onchange: move |checked| new_comment.set(checked),
                            }
                        }

                        // New user registration
                        div { class: "flex items-center justify-between py-4",
                            div { class: "space-y-0.5 pr-4",
                                p { class: "text-sm font-medium", "New user registration" }
                                p { class: "text-xs text-muted-foreground",
                                    "Get notified when a new user signs up."
                                }
                            }
                            Checkbox {
                                checked: *new_user.read(),
                                onchange: move |checked| new_user.set(checked),
                            }
                        }

                        // Payment received
                        div { class: "flex items-center justify-between py-4",
                            div { class: "space-y-0.5 pr-4",
                                p { class: "text-sm font-medium", "Payment received" }
                                p { class: "text-xs text-muted-foreground",
                                    "Get notified when a payment is successfully processed."
                                }
                            }
                            Checkbox {
                                checked: *payment_received.read(),
                                onchange: move |checked| payment_received.set(checked),
                            }
                        }

                        // Subscription cancelled
                        div { class: "flex items-center justify-between py-4",
                            div { class: "space-y-0.5 pr-4",
                                p { class: "text-sm font-medium", "Subscription cancelled" }
                                p { class: "text-xs text-muted-foreground",
                                    "Get notified when a user cancels their subscription."
                                }
                            }
                            Checkbox {
                                checked: *subscription_cancelled.read(),
                                onchange: move |checked| subscription_cancelled.set(checked),
                            }
                        }

                        // Newsletter subscriber
                        div { class: "flex items-center justify-between py-4",
                            div { class: "space-y-0.5 pr-4",
                                p { class: "text-sm font-medium", "Newsletter subscriber" }
                                p { class: "text-xs text-muted-foreground",
                                    "Get notified when someone subscribes to the newsletter."
                                }
                            }
                            Checkbox {
                                checked: *newsletter_subscriber.read(),
                                onchange: move |checked| newsletter_subscriber.set(checked),
                            }
                        }

                        // Contact form submission
                        div { class: "flex items-center justify-between py-4 last:pb-0",
                            div { class: "space-y-0.5 pr-4",
                                p { class: "text-sm font-medium", "Contact form submission" }
                                p { class: "text-xs text-muted-foreground",
                                    "Get notified when someone submits the contact form."
                                }
                            }
                            Checkbox {
                                checked: *contact_form.read(),
                                onchange: move |checked| contact_form.set(checked),
                            }
                        }
                    }
                }

                // -- Notification Channels --
                div { class: "space-y-4",
                    div {
                        h2 { class: "text-lg font-semibold", "Notification Channels" }
                        p { class: "text-sm text-muted-foreground",
                            "Enable or disable delivery channels for notifications."
                        }
                    }

                    Card { class: "p-6 divide-y divide-border",
                        // Email channel
                        div { class: "flex items-center justify-between py-4 first:pt-0",
                            div { class: "space-y-0.5 pr-4",
                                p { class: "text-sm font-medium", "Email" }
                                p { class: "text-xs text-muted-foreground",
                                    "Receive notifications via email."
                                }
                            }
                            Checkbox {
                                checked: *channel_email.read(),
                                onchange: move |checked| channel_email.set(checked),
                            }
                        }

                        // In-app channel
                        div { class: "flex items-center justify-between py-4",
                            div { class: "space-y-0.5 pr-4",
                                div { class: "flex items-center gap-2",
                                    p { class: "text-sm font-medium", "In-App" }
                                    span { class: "text-[10px] px-1.5 py-0.5 rounded-full bg-muted text-muted-foreground font-medium",
                                        "Coming Soon"
                                    }
                                }
                                p { class: "text-xs text-muted-foreground",
                                    "Receive notifications within the admin panel."
                                }
                            }
                            Checkbox {
                                checked: *channel_in_app.read(),
                                disabled: true,
                                onchange: move |checked| channel_in_app.set(checked),
                            }
                        }

                        // Webhook channel
                        div { class: "flex items-center justify-between py-4 last:pb-0",
                            div { class: "space-y-0.5 pr-4",
                                p { class: "text-sm font-medium", "Webhook" }
                                p { class: "text-xs text-muted-foreground",
                                    "Send notifications to an external endpoint via HTTP POST."
                                }
                            }
                            Checkbox {
                                checked: *channel_webhook.read(),
                                onchange: move |checked| channel_webhook.set(checked),
                            }
                        }
                    }
                }

                // -- Webhook URL Configuration --
                div { class: "space-y-4",
                    div {
                        h2 { class: "text-lg font-semibold", "Webhook Configuration" }
                        p { class: "text-sm text-muted-foreground",
                            "Configure the endpoint URL for webhook notifications."
                        }
                    }

                    Card { class: "p-6 space-y-4",
                        div { class: "grid gap-2 max-w-lg",
                            label { class: "text-sm font-medium", "Webhook Endpoint URL" }
                            input {
                                r#type: "url",
                                class: "w-full rounded-lg border border-border bg-background px-3 py-2 text-sm",
                                placeholder: "https://example.com/webhooks/notifications",
                                value: "{webhook_url}",
                                oninput: move |e| webhook_url.set(e.value()),
                            }
                            p { class: "text-xs text-muted-foreground",
                                "Notifications will be sent as POST requests with a JSON payload to this URL."
                            }
                        }

                        div { class: "flex items-center gap-3",
                            Button {
                                variant: ButtonVariant::Outline,
                                disabled: webhook_url.read().is_empty(),
                                "Test Webhook"
                            }
                            Button {
                                variant: ButtonVariant::Default,
                                disabled: webhook_url.read().is_empty(),
                                "Save Configuration"
                            }
                        }
                    }
                }
            }
        }
    }
}
