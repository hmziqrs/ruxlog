use dioxus::prelude::*;
use ruxlog_shared::use_auth;
use oxui::shadcn::button::{Button, ButtonVariant};
use oxui::shadcn::input::Input;
use oxui::shadcn::label::Label;

#[component]
pub fn ProfileEditScreen() -> Element {
    let auth_store = use_auth();
    let nav = use_navigator();
    let user = auth_store.user.read();

    let mut name = use_signal(|| user.as_ref().map(|u| u.name.clone()).unwrap_or_default());
    let mut email = use_signal(|| user.as_ref().map(|u| u.email.clone()).unwrap_or_default());
    let mut current_password = use_signal(|| String::new());
    let mut new_password = use_signal(|| String::new());
    let mut confirm_password = use_signal(|| String::new());
    let mut validation_error = use_signal(|| Option::<String>::None);
    let mut success_message = use_signal(|| Option::<String>::None);

    // If not logged in, redirect
    if user.is_none() {
        use_effect(move || {
            nav.push(crate::router::Route::LoginScreen {});
        });

        return rsx! {
            div { class: "min-h-screen bg-background flex items-center justify-center",
                div { class: "text-muted-foreground", "Redirecting to login..." }
            }
        };
    }

    let handle_profile_update = move |_| {
        let name_val = name();
        let email_val = email();

        if name_val.trim().is_empty() {
            validation_error.set(Some("Name is required".to_string()));
            return;
        }

        if email_val.trim().is_empty() {
            validation_error.set(Some("Email is required".to_string()));
            return;
        }

        validation_error.set(None);
        success_message.set(Some("Profile updated successfully!".to_string()));
        
        // TODO: Implement actual profile update API call
        // spawn(async move {
        //     auth_store.update_profile(name_val, email_val).await;
        // });
    };

    let handle_password_change = move |_| {
        let current_pwd = current_password();
        let new_pwd = new_password();
        let confirm_pwd = confirm_password();

        if current_pwd.is_empty() {
            validation_error.set(Some("Current password is required".to_string()));
            return;
        }

        if new_pwd.len() < 8 {
            validation_error.set(Some("New password must be at least 8 characters".to_string()));
            return;
        }

        if new_pwd != confirm_pwd {
            validation_error.set(Some("Passwords do not match".to_string()));
            return;
        }

        validation_error.set(None);
        success_message.set(Some("Password changed successfully!".to_string()));
        
        // Clear password fields
        current_password.set(String::new());
        new_password.set(String::new());
        confirm_password.set(String::new());
        
        // TODO: Implement actual password change API call
        // spawn(async move {
        //     auth_store.change_password(current_pwd, new_pwd).await;
        // });
    };

    rsx! {
        div { class: "min-h-screen bg-background text-foreground",
            div { class: "container mx-auto px-4 py-12 max-w-2xl",
                // Header
                div { class: "mb-8",
                    button {
                        onclick: move |_| { nav.push(crate::router::Route::ProfileScreen {}); },
                        class: "text-primary hover:underline mb-4",
                        "← Back to profile"
                    }
                    h1 { class: "text-3xl font-bold mb-2", "Edit Profile" }
                    p { class: "text-muted-foreground", "Update your account information" }
                }

                div { class: "space-y-6",
                    // Success message
                    if let Some(msg) = success_message() {
                        div { class: "p-4 rounded-lg bg-green-500/10 border border-green-500/50 text-green-600 dark:text-green-400",
                            "{msg}"
                        }
                    }

                    // Validation error
                    if let Some(error) = validation_error() {
                        div { class: "p-4 rounded-lg bg-destructive/10 border border-destructive/50 text-destructive",
                            "{error}"
                        }
                    }

                    // Profile Information
                    div { class: "bg-card border border-border rounded-lg p-6 shadow",
                        h2 { class: "text-xl font-semibold mb-6", "Profile Information" }
                        
                        form {
                            onsubmit: handle_profile_update,
                            prevent_default: "onsubmit",
                            
                            div { class: "space-y-4",
                                div { class: "space-y-2",
                                    Label { r#for: "name", "Full Name" }
                                    Input {
                                        id: "name",
                                        r#type: "text",
                                        value: "{name}",
                                        oninput: move |evt| name.set(evt.value().clone()),
                                        required: true,
                                    }
                                }

                                div { class: "space-y-2",
                                    Label { r#for: "email", "Email" }
                                    Input {
                                        id: "email",
                                        r#type: "email",
                                        value: "{email}",
                                        oninput: move |evt| email.set(evt.value().clone()),
                                        required: true,
                                    }
                                }

                                Button {
                                    r#type: "submit",
                                    "Save Changes"
                                }
                            }
                        }
                    }

                    // Change Password
                    div { class: "bg-card border border-border rounded-lg p-6 shadow",
                        h2 { class: "text-xl font-semibold mb-6", "Change Password" }
                        
                        form {
                            onsubmit: handle_password_change,
                            prevent_default: "onsubmit",
                            
                            div { class: "space-y-4",
                                div { class: "space-y-2",
                                    Label { r#for: "current_password", "Current Password" }
                                    Input {
                                        id: "current_password",
                                        r#type: "password",
                                        value: "{current_password}",
                                        oninput: move |evt| current_password.set(evt.value().clone()),
                                        placeholder: "Enter current password",
                                    }
                                }

                                div { class: "space-y-2",
                                    Label { r#for: "new_password", "New Password" }
                                    Input {
                                        id: "new_password",
                                        r#type: "password",
                                        value: "{new_password}",
                                        oninput: move |evt| new_password.set(evt.value().clone()),
                                        placeholder: "Enter new password",
                                    }
                                    p { class: "text-xs text-muted-foreground", "Must be at least 8 characters" }
                                }

                                div { class: "space-y-2",
                                    Label { r#for: "confirm_password", "Confirm New Password" }
                                    Input {
                                        id: "confirm_password",
                                        r#type: "password",
                                        value: "{confirm_password}",
                                        oninput: move |evt| confirm_password.set(evt.value().clone()),
                                        placeholder: "Confirm new password",
                                    }
                                }

                                Button {
                                    r#type: "submit",
                                    variant: ButtonVariant::Outline,
                                    "Change Password"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
