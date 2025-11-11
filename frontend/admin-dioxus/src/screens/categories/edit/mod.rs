use dioxus::prelude::*;

use crate::components::sonner::{Action, ToastOptions};
use crate::components::{FormTwoColumnSkeleton, PageHeader};
use crate::containers::{CategoryForm, CategoryFormContainer};
use crate::hooks::{use_category_view, use_state_frame_map_toast, StateFrameToastConfig};
use crate::router::Route;
use crate::store::use_categories;
use crate::ui::shadcn::{Button, ButtonVariant};

#[component]
pub fn CategoriesEditScreen(id: i32) -> Element {
    let state = use_category_view(id);
    let nav = use_navigator();
    let categories = use_categories();
    let is_loading = state.is_loading;
    let is_failed = state.is_failed;
    let message = state.message.clone();
    let cat_opt = state.category.clone();

    let toast_cfg = StateFrameToastConfig {
        loading_title: "Saving category...".into(),
        success_title: Some("Category updated successfully".into()),
        success_options: ToastOptions::default().with_action(Some(Action::with_on_click(
            "View Categories".into(),
            Callback::new(move |_| {
                nav.push(Route::CategoriesListScreen {});
            }),
        ))),
        error_title: Some("Failed to update category".into()),
        error_options: ToastOptions::default().with_action(Some(Action::with_on_click(
            "Retry".into(),
            {
                let categories = categories;
                Callback::new(move |_| {
                    if let Some(payload) = categories
                        .edit
                        .peek()
                        .get(&id)
                        .and_then(|frame| frame.meta.clone())
                    {
                        let categories = categories;
                        spawn(async move {
                            categories.edit(id, payload).await;
                        });
                    }
                })
            },
        ))),
        ..Default::default()
    };
    use_state_frame_map_toast(&categories.edit, id, toast_cfg);

    // Compute initial form state from loaded category
    let initial_form: Option<CategoryForm> = cat_opt.clone().map(|c| CategoryForm {
        name: c.name.clone(),
        slug: c.slug.clone(),
        description: c.description.unwrap_or_default(),
        color: c.color.clone(),
        custom_text_color: true, // respect existing text color; container computes contrast when not custom
        text_color: c.text_color.clone(),
        active: c.is_active,
        logo_blob_url: c.logo.as_ref().map(|m| m.file_url.clone()),
        logo_media_id: c.logo_id,
        logo_filename: c.logo.as_ref().map(|m| m.object_key.clone()),
        logo_size: c.logo.as_ref().map(|m| m.size),
        cover_blob_url: c.cover.as_ref().map(|m| m.file_url.clone()),
        cover_media_id: c.cover_id,
        cover_filename: c.cover.as_ref().map(|m| m.object_key.clone()),
        cover_size: c.cover.as_ref().map(|m| m.size),
        parent_id: c.parent_id.map(|v| v.to_string()).unwrap_or_default(),
    });

    rsx! {
        div { class: "min-h-screen bg-transparent text-foreground",
            PageHeader {
                title: "Edit Category".to_string(),
                description: "Update category details, colors, and visibility.".to_string(),
                actions: Some(rsx! {
                    Button {
                        variant: ButtonVariant::Outline,
                        onclick: move |_| { nav.push(Route::CategoriesListScreen {}); },
                        "Back to Categories"
                    }
                }),
            }

            div { class: "container mx-auto px-4 py-10 md:py-12 space-y-4",
                if is_failed {
                    div { class: "rounded-md border border-red-200 bg-red-50 p-3 text-red-700 dark:border-red-900/40 dark:bg-red-900/20 dark:text-red-300",
                        span { class: "text-sm", "Failed to load category." }
                        if let Some(msg) = message { span { class: "ml-1 text-sm opacity-80", "{msg}" } }
                        Button {
                            class: "ml-3",
                            onclick: move |_| {
                                let categories = categories;
                                spawn(async move {
                                    categories.view(id).await;
                                });
                            },
                            "Retry"
                        }
                    }
                }

                if is_loading && initial_form.is_none() {
                    FormTwoColumnSkeleton {}
                } else if let Some(initial) = initial_form.clone() {
                    CategoryFormContainer {
                        title: Some("Edit Category".to_string()),
                        submit_label: Some("Save Changes".to_string()),
                        initial: Some(initial.clone()),
                        on_submit: move |val: CategoryForm| {
                            let payload = val.to_edit_payload();
                            let categories = categories;
                            spawn(async move {
                                categories.edit(id, payload).await;
                            });
                        },
                    }
                }
            }
        }
    }
}
