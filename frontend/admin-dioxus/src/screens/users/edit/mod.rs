use dioxus::prelude::*;

use crate::components::sonner::{Action, ToastOptions};
use crate::components::{FormTwoColumnSkeleton, PageHeader};
use crate::containers::{UserForm, UserFormContainer};
use crate::hooks::{use_state_frame_map_toast, use_user_view, StateFrameToastConfig};
use crate::router::Route;
use crate::store::use_user;
use crate::ui::shadcn::{Button, ButtonVariant};

#[component]
pub fn UsersEditScreen(id: i32) -> Element {
    let state = use_user_view(id);
    let nav = use_navigator();
    let users = use_user();
    let is_loading = state.is_loading;
    let is_failed = state.is_failed;
    let message = state.message.clone();
    let user_opt = state.user.clone();

    let toast_cfg = StateFrameToastConfig {
        loading_title: "Saving user...".into(),
        success_title: Some("User updated successfully".into()),
        success_options: ToastOptions::default().with_action(Some(Action::with_on_click(
            "View Users".into(),
            Callback::new(move |_| {
                nav.push(Route::UsersListScreen {});
            }),
        ))),
        error_title: Some("Failed to update user".into()),
        error_options: ToastOptions::default().with_action(Some(Action::with_on_click(
            "Retry".into(),
            {
                let users = users;
                Callback::new(move |_| {
                    if let Some(payload) = users
                        .edit
                        .peek()
                        .get(&id)
                        .and_then(|frame| frame.meta.clone())
                    {
                        let users = users;
                        spawn(async move {
                            users.edit(id, payload).await;
                        });
                    }
                })
            },
        ))),
        ..Default::default()
    };
    use_state_frame_map_toast(&users.edit, id, toast_cfg);

    // Compute initial form state from loaded user
    let initial_form: Option<UserForm> = user_opt.clone().map(|u| UserForm {
        name: u.name.clone(),
        email: u.email.clone(),
        role: u.role.to_string(),
        is_verified: u.is_verified,
        password: None,         // Don't prefill password for edit
        confirm_password: None, // Don't prefill confirm password for edit
        avatar_id: u.avatar.as_ref().map(|a| a.id),
        avatar_blob_url: None,             // No blob URL when editing existing
        existing_avatar: u.avatar.clone(), // Pass existing avatar for display
        is_update: true,
    });

    rsx! {
        div { class: "min-h-screen bg-transparent text-foreground",
            PageHeader {
                title: "Edit User".to_string(),
                description: "Update user details, role, and permissions.".to_string(),
                actions: Some(rsx! {
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| { nav.push(Route::UsersListScreen {}); },
                        "Back to Users"
                    }
                }),
            }

            div { class: "container mx-auto px-4 py-10 md:py-12 space-y-4",
                if is_failed {
                    div { class: "rounded-md border border-red-200 bg-red-50 p-3 text-red-700 dark:border-red-900/40 dark:bg-red-900/20 dark:text-red-300",
                        span { class: "text-sm", "Failed to load user." }
                        if let Some(msg) = message { span { class: "ml-1 text-sm opacity-80", "{msg}" } }
                        Button {
                            class: "ml-3",
                            onclick: move |_| {
                                let users = users;
                                spawn(async move {
                                    users.view(id).await;
                                });
                            },
                            "Retry"
                        }
                    }
                }

                if is_loading && initial_form.is_none() {
                    FormTwoColumnSkeleton {}
                } else if let Some(initial) = initial_form.clone() {
                    UserFormContainer {
                        title: Some("Edit User".to_string()),
                        submit_label: Some("Save Changes".to_string()),
                        initial: Some(initial.clone()),
                        on_submit: move |val: UserForm| {
                            let payload = val.to_edit_payload();
                            let users = users;
                            spawn(async move {
                                users.edit(id, payload).await;
                            });
                        },
                    }
                }
            }
        }
    }
}
