use dioxus::prelude::*;
use ruxlog_shared::store::{use_auth, TwoFactorVerifyPayload, UserSession};

use crate::containers::page_header::PageHeader;
use oxui::components::form::input::SimpleInput;
use oxui::shadcn::button::{Button, ButtonVariant};

#[component]
pub fn ProfileSecurityScreen() -> Element {
    let auth = use_auth();
    let mut verification_code = use_signal(|| "".to_string());
    let mut disable_code = use_signal(|| "".to_string());

    // Fetch sessions on mount
    use_effect(move || {
        spawn(async move {
            auth.list_sessions().await;
        });
    });

    let sessions: Vec<UserSession> = auth.sessions.read().data.clone().unwrap_or_default();
    let two_factor = auth.two_factor.read();
    let setup_payload = two_factor.data.clone().unwrap_or(None);

    let setup = move |_| {
        let auth = auth;
        spawn(async move {
            auth.setup_2fa().await;
        });
    };

    let verify = move |e: FormEvent| {
        e.prevent_default();
        let auth = auth;
        let body = TwoFactorVerifyPayload {
            code: verification_code(),
        };
        spawn(async move {
            auth.verify_2fa(body).await;
        });
    };

    let disable = move |e: FormEvent| {
        e.prevent_default();
        let auth = auth;
        let body = TwoFactorVerifyPayload {
            code: disable_code(),
        };
        spawn(async move {
            auth.disable_2fa(body).await;
        });
    };

    let refresh_sessions = move |_| {
        let auth = auth;
        spawn(async move {
            auth.list_sessions().await;
        });
    };

    rsx! {
        div { class: "min-h-screen bg-transparent",
            PageHeader {
                title: "Security".to_string(),
                description: "Manage 2FA and active sessions".to_string(),
                actions: Some(rsx!{
                    Button {
                        onclick: refresh_sessions,
                        "Refresh"
                    }
                }),
                class: None,
                embedded: false,
            }

            div { class: "container mx-auto px-4 py-8 md:py-12",
                div { class: "grid gap-4 md:grid-cols-2",
                    div { class: "border border-border rounded-md p-4 space-y-3",
                        div { class: "flex items-center gap-2",
                            h3 { class: "text-lg font-semibold", "Two-Factor Auth" }
                        }
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: setup,
                            "Setup 2FA"
                        }
                        if let Some(setup_info) = setup_payload.clone() {
                            div { class: "space-y-2",
                                p { class: "text-sm text-muted-foreground", "Scan QR code or use the secret below." }
                                img { class: "w-40 h-40 border border-border rounded-md", src: setup_info.qr_code_url }
                                div { class: "font-mono text-xs break-all border border-border rounded-md px-3 py-2", "{setup_info.secret}" }
                                if !setup_info.backup_codes.is_empty() {
                                    div { class: "space-y-1",
                                        h4 { class: "text-sm font-semibold", "Backup Codes" }
                                        ul { class: "grid grid-cols-2 gap-1 text-xs font-mono",
                                            for code in setup_info.backup_codes {
                                                li { class: "rounded border border-border px-2 py-1", "{code}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        form { class: "space-y-2", onsubmit: verify,
                            label { class: "text-sm font-medium", "Verify code" }
                            SimpleInput {
                                placeholder: Some("123456".to_string()),
                                value: verification_code(),
                                oninput: move |value| verification_code.set(value),
                                class: Some("text-sm".to_string()),
                            }
                            Button { r#type: "submit", "Verify" }
                        }

                        form { class: "space-y-2", onsubmit: disable,
                            label { class: "text-sm font-medium", "Disable with code" }
                            SimpleInput {
                                placeholder: Some("123456".to_string()),
                                value: disable_code(),
                                oninput: move |value| disable_code.set(value),
                                class: Some("text-sm".to_string()),
                            }
                            Button { variant: ButtonVariant::Outline, r#type: "submit", "Disable 2FA" }
                        }
                    }

                    div { class: "border border-border rounded-md p-4 space-y-3",
                        div { class: "flex items-center gap-2",
                            h3 { class: "text-lg font-semibold", "Active Sessions" }
                        }
                        if sessions.is_empty() {
                            p { class: "text-sm text-muted-foreground", "No active sessions." }
                        } else {
                            div { class: "space-y-2",
                                for session in sessions {
                                    SessionCard { key: "{session.id}", session }
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
fn SessionCard(session: UserSession) -> Element {
    let auth = use_auth();
    rsx! {
        div { class: "rounded-md border border-border px-3 py-2 text-sm flex flex-col gap-1",
            div { class: "font-medium", "Session {session.id}" }
            div { class: "text-muted-foreground", "IP: {session.ip.clone().unwrap_or_else(|| \"n/a\".to_string())}" }
            div { class: "text-muted-foreground", "Agent: {session.user_agent.clone().unwrap_or_else(|| \"n/a\".to_string())}" }
            div { class: "text-muted-foreground", "Last active: {session.last_active}" }
            div { class: "mt-2 flex gap-2",
                Button {
                    variant: ButtonVariant::Outline,
                    class: "h-8 px-3 text-xs",
                    onclick: move |_| {
                        let auth = auth;
                        let id = session.id.clone();
                        spawn(async move { auth.terminate_session(id).await; });
                    },
                    "Terminate"
                }
            }
        }
    }
}
