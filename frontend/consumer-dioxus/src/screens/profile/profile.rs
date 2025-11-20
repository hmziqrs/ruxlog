use dioxus::prelude::*;
use hmziq_dioxus_free_icons::icons::ld_icons::{LdMail, LdUser};
use hmziq_dioxus_free_icons::Icon;
use oxui::shadcn::button::{Button, ButtonVariant};
use ruxlog_shared::use_auth;

#[component]
pub fn ProfileScreen() -> Element {
    let auth_store = use_auth();
    let nav = use_navigator();
    let user = auth_store.user.read();

    if let Some(user) = &*user {

        rsx! {
            div { class: "min-h-screen bg-background text-foreground",
                div { class: "container mx-auto px-4 py-12 max-w-4xl",
                    // Header
                    div { class: "mb-8",
                        h1 { class: "text-3xl font-bold mb-2", "Profile" }
                        p { class: "text-muted-foreground", "Manage your account settings and preferences" }
                    }

                    // Profile card
                    div { class: "bg-card border border-border rounded-lg shadow-lg overflow-hidden",
                        // Cover/Header section
                        div { class: "h-32 bg-gradient-to-r from-primary/20 via-primary/10 to-primary/20" }

                        div { class: "px-8 pb-8",
                            // Avatar and basic info
                            div { class: "-mt-16 mb-6",
                                div { class: "w-32 h-32 rounded-full bg-primary/20 border-4 border-card flex items-center justify-center",
                                    if let Some(avatar) = &user.avatar {
                                        img {
                                            src: "{avatar.file_url}",
                                            alt: "{user.name}",
                                            class: "w-full h-full rounded-full object-cover"
                                        }
                                    } else {
                                        span { class: "text-4xl font-bold text-primary",
                                            "{user.name.chars().next().unwrap_or('U').to_uppercase()}"
                                        }
                                    }
                                }
                            }

                            div { class: "space-y-6",
                                // Name
                                div { class: "flex items-start gap-4",
                                    div { class: "w-12 h-12 rounded-lg bg-primary/10 flex items-center justify-center shrink-0",
                                        Icon { icon: LdUser, class: "w-6 h-6 text-primary" }
                                    }
                                    div { class: "flex-1",
                                        p { class: "text-sm text-muted-foreground mb-1", "Name" }
                                        p { class: "text-lg font-medium", "{user.name}" }
                                    }
                                }

                                // Email
                                div { class: "flex items-start gap-4",
                                    div { class: "w-12 h-12 rounded-lg bg-primary/10 flex items-center justify-center shrink-0",
                                        Icon { icon: LdMail, class: "w-6 h-6 text-primary" }
                                    }
                                    div { class: "flex-1",
                                        p { class: "text-sm text-muted-foreground mb-1", "Email" }
                                        p { class: "text-lg font-medium", "{user.email}" }
                                        if user.is_verified {
                                            span { class: "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-green-500/10 text-green-600 dark:text-green-400 mt-1",
                                                "✓ Verified"
                                            }
                                        } else {
                                            span { class: "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-yellow-500/10 text-yellow-600 dark:text-yellow-400 mt-1",
                                                "⚠ Not verified"
                                            }
                                        }
                                    }
                                }

                                // Actions
                                div { class: "pt-6 border-t border-border flex gap-3",
                                    Button {
                                        onclick: move |_| {
                                            nav.push(crate::router::Route::ProfileEditScreen {});
                                        },
                                        class: "flex-1",
                                        "Edit Profile"
                                    }
                                    Button {
                                        variant: ButtonVariant::Outline,
                                        onclick: move |_| {
                                            spawn(async move {
                                                auth_store.logout().await;
                                                nav.push(crate::router::Route::LoginScreen {});
                                            });
                                        },
                                        "Sign Out"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        // Not logged in - redirect to login
        use_effect(move || {
            nav.push(crate::router::Route::LoginScreen {});
        });

        rsx! {
            div { class: "min-h-screen bg-background flex items-center justify-center",
                div { class: "text-muted-foreground", "Redirecting to login..." }
            }
        }
    }
}
