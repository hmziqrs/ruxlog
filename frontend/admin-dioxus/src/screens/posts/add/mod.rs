use dioxus::prelude::*;

use crate::components::sonner::{Action, ToastOptions};
use crate::components::PageHeader;
use crate::containers::BlogFormContainer;
use crate::hooks::{use_state_frame_toast, StateFrameToastConfig};
use crate::router::Route;
use crate::store::use_post;
use crate::ui::shadcn::{Button, ButtonVariant};

#[component]
pub fn PostsAddScreen() -> Element {
    let posts = use_post();
    let nav = use_navigator();

    // Wire StateFrame->Sonner toast for add flow
    let cfg = StateFrameToastConfig {
        loading_title: "Creating post...".into(),
        success_title: Some("Post created successfully".into()),
        success_options: ToastOptions::default().with_action(Some(Action::with_on_click(
            "View Posts".into(),
            Callback::new(move |_| {
                nav.push(Route::PostsListScreen {});
            }),
        ))),
        error_title: Some("Failed to create post".into()),
        error_options: ToastOptions::default().with_action(Some(Action::with_on_click(
            "Retry".into(),
            Callback::new(move |_| {
                let payload = posts.add.peek().meta.clone();
                spawn(async move {
                    posts.add(payload.unwrap()).await;
                });
            }),
        ))),
        ..Default::default()
    };
    use_state_frame_toast(&posts.add, cfg);

    rsx! {
        // Page wrapper
        div { class: "min-h-screen bg-transparent text-foreground",
            // Unified autonomous header
            PageHeader {
                title: "Create Post".to_string(),
                description: "Write and publish your blog post. Add rich content, images, and categorize your post.".to_string(),
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
                BlogFormContainer { post_id: None }
            }
        }
    }
}
