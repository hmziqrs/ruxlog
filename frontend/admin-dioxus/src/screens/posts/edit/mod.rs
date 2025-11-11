use dioxus::prelude::*;

use crate::components::sonner::{Action, ToastOptions};
use crate::components::PageHeader;
use crate::containers::BlogFormContainer;
use crate::hooks::{use_state_frame_map_toast, StateFrameToastConfig};
use crate::router::Route;
use crate::store::use_post;
use crate::ui::shadcn::{Button, ButtonVariant};

#[component]
pub fn PostsEditScreen(id: i32) -> Element {
    let posts = use_post();
    let nav = use_navigator();

    // Wire StateFrame->Sonner toast for edit flow
    let toast_cfg = StateFrameToastConfig {
        loading_title: "Saving post...".into(),
        success_title: Some("Post updated successfully".into()),
        success_options: ToastOptions::default().with_action(Some(Action::with_on_click(
            "View Posts".into(),
            Callback::new(move |_| {
                nav.push(Route::PostsListScreen {});
            }),
        ))),
        error_title: Some("Failed to update post".into()),
        error_options: ToastOptions::default().with_action(Some(Action::with_on_click(
            "Retry".into(),
            {
                let posts = posts;
                Callback::new(move |_| {
                    if let Some(payload) = posts
                        .edit
                        .peek()
                        .get(&id)
                        .and_then(|frame| frame.meta.clone())
                    {
                        let posts = posts;
                        spawn(async move {
                            posts.edit(id, payload).await;
                        });
                    }
                })
            },
        ))),
        ..Default::default()
    };
    use_state_frame_map_toast(&posts.edit, id, toast_cfg);

    rsx! {
        // Page wrapper
        div { class: "min-h-screen bg-transparent text-foreground",
            // Unified autonomous header
            PageHeader {
                title: "Edit Post".to_string(),
                description: "Update your post content, metadata, and publishing settings.".to_string(),
                actions: Some(rsx! {
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| { nav.push(Route::PostsListScreen {}); },
                        "Back to Posts"
                    }
                }),
            }

            // Content: render blog form container
            div { class: "container mx-auto px-4 py-10 md:py-12",
                BlogFormContainer { post_id: Some(id) }
            }
        }
    }
}
