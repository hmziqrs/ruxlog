use dioxus::prelude::*;
use ruxlog_shared::store::{
    use_auth, TwoFactorVerifyPayload, UserSession,
};

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
        let body = TwoFactorVerifyPayload { code: verification_code() };
        spawn(async move { auth.verify_2fa(body).await; });
    };

    let disable = move |e: FormEvent| {
        e.prevent_default();
        let auth = auth;
        let body = TwoFactorVerifyPayload { code: disable_code() };
        spawn(async move { auth.disable_2fa(body).await; });
    };

    let refresh_sessions = move |_| {
        let auth = auth;
        spawn(async move { auth.list_sessions().await; });
    };

    rsx! {
        div { class: "p-6 space-y-6",
            div { class: "flex items-center justify-between",
                div {
                    h2 { class: "text-2xl font-semibold", "Security" }
                    p { class: "text-sm text-muted-foreground", "Manage 2FA and active sessions." }
                }
                button {
                    class: "inline-flex items-center gap-2 rounded-md border border-border px-3 py-2 text-sm hover:bg-accent",
                    onclick: refresh_sessions,
                    "Refresh"
                }
            }

            div { class: "grid gap-4 md:grid-cols-2",
                div { class: "border border-border rounded-md p-4 space-y-3",
                    div { class: "flex items-center gap-2",
                        h3 { class: "text-lg font-semibold", "Two-Factor Auth" }
                    }
                    button {
                        class: "rounded-md border border-border px-3 py-2 text-sm font-medium hover:bg-accent",
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
                        input {
                            class: "w-full rounded-md border border-border px-3 py-2 text-sm",
                            placeholder: "123456",
                            value: "{verification_code}",
                            oninput: move |e| verification_code.set(e.value()),
                        }
                        button { class: "rounded-md bg-primary px-3 py-2 text-sm text-primary-foreground hover:opacity-90", r#type: "submit", "Verify" }
                    }

                    form { class: "space-y-2", onsubmit: disable,
                        label { class: "text-sm font-medium", "Disable with code" }
                        input {
                            class: "w-full rounded-md border border-border px-3 py-2 text-sm",
                            placeholder: "123456",
                            value: "{disable_code}",
                            oninput: move |e| disable_code.set(e.value()),
                        }
                        button { class: "rounded-md border border-border px-3 py-2 text-sm hover:bg-accent", r#type: "submit", "Disable 2FA" }
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
                button {
                    class: "rounded-md border border-border px-3 py-1 text-xs hover:bg-accent",
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
