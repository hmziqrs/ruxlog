use dioxus::prelude::*;

use crate::components::sonner::{Action, ToastOptions};
use crate::components::PageHeader;
use crate::containers::{UserForm, UserFormContainer};
use crate::hooks::{use_state_frame_toast, StateFrameToastConfig};
use crate::router::Route;
use crate::store::use_user;
use crate::ui::shadcn::{Button, ButtonVariant};

#[component]
pub fn UsersAddScreen() -> Element {
    let users = use_user();
    let nav = use_navigator();

    // Wire StateFrame->Sonner toast for add flow
    let cfg = StateFrameToastConfig {
        loading_title: "Creating user...".into(),
        success_title: Some("User created successfully".into()),
        success_options: ToastOptions::default().with_action(Some(Action::with_on_click(
            "View Users".into(),
            Callback::new(move |_| {
                nav.push(Route::UsersListScreen {});
            }),
        ))),
        error_title: Some("Failed to create user".into()),
        error_options: ToastOptions::default().with_action(Some(Action::with_on_click(
            "Retry".into(),
            Callback::new(move |_| {
                let payload = users.add.peek().meta.clone();
                spawn(async move {
                    users.add(payload.unwrap()).await;
                });
            }),
        ))),
        ..Default::default()
    };
    use_state_frame_toast(&users.add, cfg);

    rsx! {
        // Page wrapper
        div { class: "min-h-screen bg-transparent text-foreground",
            // Unified autonomous header
            PageHeader {
                title: "Create User".to_string(),
                description: "Add a new user account with role and permissions.".to_string(),
                actions: Some(rsx! {
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| { nav.push(Route::UsersListScreen {}); },
                        "Back to Users"
                    }
                }),
            }

            // Content: render reusable form component; submission handled here
            div { class: "container mx-auto px-4 py-10 md:py-12",
                    UserFormContainer {
                        title: Some("New User".to_string()),
                        submit_label: Some("Create User".to_string()),
                        on_submit: move |val: UserForm| {
                            let payload = val.to_add_payload();
                            let users = users;
                            spawn(async move {
                                users.add(payload).await;
                            });
                        },
                }
            }
        }
    }
}
