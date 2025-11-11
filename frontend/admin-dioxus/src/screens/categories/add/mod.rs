use dioxus::prelude::*;

use crate::components::sonner::{Action, ToastOptions};
use crate::components::PageHeader;
use crate::containers::{CategoryForm, CategoryFormContainer};
use crate::hooks::{use_state_frame_toast, StateFrameToastConfig};
use crate::router::Route;
use crate::store::use_categories;
use crate::ui::shadcn::{Button, ButtonVariant};

#[component]
pub fn CategoriesAddScreen() -> Element {
    let categories = use_categories();
    let nav = use_navigator();

    // Wire StateFrame->Sonner toast for add flow
    let cfg = StateFrameToastConfig {
        loading_title: "Creating category...".into(),
        success_title: Some("Category created successfully".into()),
        success_options: ToastOptions::default().with_action(Some(Action::with_on_click(
            "View Categories".into(),
            Callback::new(move |_| {
                nav.push(Route::CategoriesListScreen {});
            }),
        ))),
        error_title: Some("Failed to create category".into()),
        error_options: ToastOptions::default().with_action(Some(Action::with_on_click(
            "Retry".into(),
            Callback::new(move |_| {
                let payload = categories.add.peek().meta.clone();
                spawn(async move {
                    categories.add(payload.unwrap()).await;
                });
            }),
        ))),
        ..Default::default()
    };
    use_state_frame_toast(&categories.add, cfg);

    rsx! {
        // Page wrapper
        div { class: "min-h-screen bg-transparent text-foreground",
            // Unified autonomous header
            PageHeader {
                title: "Create Category".to_string(),
                description: "Define category details, colors, and visibility.".to_string(),
                actions: Some(rsx! {
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| { nav.push(Route::CategoriesListScreen {}); },
                        "Back to Categories"
                    }
                }),
            }

            // Content: render reusable form component; submission handled here
            div { class: "container mx-auto px-4 py-10 md:py-12",
                    CategoryFormContainer {
                        title: Some("New Category".to_string()),
                        submit_label: Some("Create Category".to_string()),
                        on_submit: move |val: CategoryForm| {
                            let payload = val.to_add_payload();
                            let categories = categories;
                            spawn(async move {
                                categories.add(payload).await;
                            });
                        },
                }
            }
        }
    }
}
