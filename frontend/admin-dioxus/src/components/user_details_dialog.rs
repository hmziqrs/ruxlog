use dioxus::prelude::*;

use crate::components::UserAvatar;
use crate::router::Route;
use crate::store::User;
use crate::ui::custom::AppPortal;
use crate::ui::shadcn::{Badge, BadgeVariant, Button, ButtonVariant};
use crate::utils::dates::format_short_date_dt;
use hmziq_dioxus_free_icons::{icons::ld_icons::LdX, Icon};

#[derive(Props, Clone, PartialEq)]
pub struct UserDetailsDialogProps {
    pub is_open: Signal<bool>,
    pub user: Option<User>,
}

/// A dialog component to display user details in a read-only view
#[component]
pub fn UserDetailsDialog(mut props: UserDetailsDialogProps) -> Element {
    if !*props.is_open.read() {
        return rsx! {};
    }

    let user = match &props.user {
        Some(u) => u.clone(),
        None => return rsx! {},
    };

    let nav = use_navigator();
    let user_id = user.id;

    let handle_close = move |_| {
        props.is_open.set(false);
    };

    let handle_edit = move |_| {
        props.is_open.set(false);
        nav.push(Route::UsersEditScreen { id: user_id });
    };

    rsx! {
        AppPortal {
            // Backdrop
            div {
                class: "fixed inset-0 z-50 bg-black/50",
                onclick: handle_close,
            }

            // Dialog content
            div {
                class: "fixed top-[50%] left-[50%] z-50 grid w-full max-w-2xl translate-x-[-50%] translate-y-[-50%] gap-4 rounded-lg border border-zinc-200 dark:border-zinc-800 bg-background p-6 shadow-lg duration-200",
                onclick: move |e| e.stop_propagation(),

                // Header with close button
                div { class: "flex items-start justify-between mb-4",
                    div {
                        h2 { class: "text-xl font-semibold leading-none", "User Details" }
                        p { class: "text-sm text-muted-foreground mt-1.5", "View user profile information" }
                    }
                    button {
                        onclick: handle_close,
                        class: "rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none",
                        div { class: "w-4 h-4",
                            Icon { icon: LdX {} }
                        }
                        span { class: "sr-only", "Close" }
                    }
                }

                // User avatar and basic info section
                div { class: "flex items-start gap-4 pb-4 border-b border-zinc-200 dark:border-zinc-800",
                    div { class: "flex-shrink-0",
                        UserAvatar {
                            name: user.name.clone(),
                            avatar: user.avatar.clone(),
                            size: "w-20 h-20".to_string(),
                            text_size: "text-2xl".to_string(),
                        }
                    }
                    div { class: "flex-1 min-w-0",
                        h3 { class: "text-lg font-semibold truncate", "{user.name}" }
                        p { class: "text-sm text-muted-foreground truncate mt-1", "{user.email}" }
                        div { class: "flex flex-wrap gap-2 mt-3",
                            {match user.role {
                                crate::store::UserRole::SuperAdmin => rsx! {
                                    Badge { class: "bg-purple-100 text-purple-800 border-purple-200 dark:bg-purple-900/20 dark:text-purple-400", "Super Admin" }
                                },
                                crate::store::UserRole::Admin => rsx! {
                                    Badge { class: "bg-red-100 text-red-800 border-red-200 dark:bg-red-900/20 dark:text-red-400", "Admin" }
                                },
                                crate::store::UserRole::Moderator => rsx! {
                                    Badge { class: "bg-orange-100 text-orange-800 border-orange-200 dark:bg-orange-900/20 dark:text-orange-400", "Moderator" }
                                },
                                crate::store::UserRole::Author => rsx! {
                                    Badge { class: "bg-blue-100 text-blue-800 border-blue-200 dark:bg-blue-900/20 dark:text-blue-400", "Author" }
                                },
                                crate::store::UserRole::User => rsx! {
                                    Badge { variant: BadgeVariant::Secondary, class: "bg-gray-100 text-gray-800 border-gray-200 dark:bg-gray-900/20 dark:text-gray-400", "User" }
                                },
                            }}
                        }
                    }
                }

                // Details grid
                div { class: "grid gap-4 py-4",
                    // Verification Status
                    div { class: "grid grid-cols-3 gap-4 items-center",
                        label { class: "text-sm font-medium", "Verification Status" }
                        div { class: "col-span-2",
                            if user.is_verified {
                                Badge { class: "bg-green-100 text-green-800 border-green-200 dark:bg-green-900/20 dark:text-green-400", "Verified" }
                            } else {
                                Badge { variant: BadgeVariant::Secondary, class: "bg-yellow-100 text-yellow-800 border-yellow-200 dark:bg-yellow-900/20 dark:text-yellow-400", "Unverified" }
                            }
                        }
                    }

                    // 2FA Status
                    div { class: "grid grid-cols-3 gap-4 items-center",
                        label { class: "text-sm font-medium", "Two-Factor Authentication" }
                        div { class: "col-span-2",
                            if user.two_fa_enabled {
                                Badge { class: "bg-green-100 text-green-800 border-green-200 dark:bg-green-900/20 dark:text-green-400", "Enabled" }
                            } else {
                                Badge { variant: BadgeVariant::Secondary, class: "bg-gray-100 text-gray-800 border-gray-200 dark:bg-gray-900/20 dark:text-gray-400", "Disabled" }
                            }
                        }
                    }

                    // User ID
                    div { class: "grid grid-cols-3 gap-4 items-center",
                        label { class: "text-sm font-medium", "User ID" }
                        div { class: "col-span-2",
                            span { class: "text-sm text-muted-foreground font-mono", "{user.id}" }
                        }
                    }

                    // Created At
                    div { class: "grid grid-cols-3 gap-4 items-center",
                        label { class: "text-sm font-medium", "Created" }
                        div { class: "col-span-2",
                            span { class: "text-sm text-muted-foreground", "{format_short_date_dt(&user.created_at)}" }
                        }
                    }

                    // Updated At
                    div { class: "grid grid-cols-3 gap-4 items-center",
                        label { class: "text-sm font-medium", "Last Updated" }
                        div { class: "col-span-2",
                            span { class: "text-sm text-muted-foreground", "{format_short_date_dt(&user.updated_at)}" }
                        }
                    }
                }

                // Footer actions
                div { class: "flex justify-end gap-2 pt-4 border-t border-zinc-200 dark:border-zinc-800",
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: handle_close,
                        "Close"
                    }
                    Button {
                        onclick: handle_edit,
                        "Edit User"
                    }
                }
            }
        }
    }
}
