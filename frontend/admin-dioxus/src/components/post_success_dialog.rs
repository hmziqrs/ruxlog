use crate::router::Route;
use crate::ui::custom::AppPortal;
use crate::ui::shadcn::{Button, ButtonVariant};
use dioxus::logger::tracing;
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::{icons::ld_icons::LdX, Icon};

#[component]
pub fn PostSuccessDialog(
    is_open: Signal<bool>,
    post_id: ReadSignal<Option<i32>>,
    is_new_post: bool,
) -> Element {
    tracing::info!(
        "Rendering PostSuccessDialog, is_new_post: {:?} {:?}",
        is_open,
        post_id()
    );

    if !*is_open.read() {
        return rsx! {};
    }

    let nav = use_navigator();

    let handle_new_post = move |_| {
        is_open.set(false);
        nav.replace(Route::PostsAddScreen {});
    };

    let handle_edit_post = move |_| {
        if let Some(id) = post_id() {
            is_open.set(false);
            nav.replace(Route::PostsEditScreen { id });
        }
    };

    let handle_view_post = move |_| {
        if let Some(id) = post_id() {
            is_open.set(false);
            nav.push(Route::PostsViewScreen { id });
        }
    };

    let handle_back_to_list = move |_| {
        is_open.set(false);
        nav.push(Route::PostsListScreen {});
    };

    let handle_close = move |_| {
        is_open.set(false);
    };

    rsx! {
        AppPortal {
            z_index: "1100",
            div {
                class: "fixed inset-0 bg-black/50",
                onclick: handle_close,
            }

            div {
                class: "fixed left-[50%] top-[50%] translate-x-[-50%] translate-y-[-50%] w-full max-w-2xl",
                onclick: move |e| e.stop_propagation(),

                div {
                    class: "bg-background rounded-lg border p-6 shadow-lg",
                    div { class: "flex items-start justify-between mb-4",
                        div {
                            h2 {
                                class: "text-lg font-semibold",
                                if is_new_post {
                                    "Post Created Successfully!"
                                } else {
                                    "Post Updated Successfully!"
                                }
                            }
                            p {
                                class: "text-sm text-muted-foreground mt-1",
                                if is_new_post {
                                    "Your post has been created. What would you like to do next?"
                                } else {
                                    "Your changes have been saved. What would you like to do next?"
                                }
                            }
                        }
                        button {
                            onclick: handle_close,
                            class: "rounded-xs opacity-70 hover:opacity-100 transition-opacity",
                            Icon { icon: LdX, width: 20, height: 20 }
                        }
                    }
                    div { class: "mt-6",
                        div { class: "flex flex-col sm:flex-row justify-end gap-2 mb-3",
                            Button {
                                variant: ButtonVariant::Default,
                                onclick: handle_new_post,
                                class: "flex-1",
                                "New Post"
                            }
                        }
                        if post_id().is_some() {
                            div { class: "flex flex-col sm:flex-row justify-end gap-2",
                                Button {
                                    variant: ButtonVariant::Outline,
                                    onclick: handle_edit_post,
                                    class: "flex-1 sm:flex-initial sm:min-w-[120px]",
                                    "Edit Post"
                                }
                                Button {
                                    variant: ButtonVariant::Outline,
                                    onclick: handle_view_post,
                                    class: "flex-1 sm:flex-initial sm:min-w-[120px]",
                                    "View Post"
                                }
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    onclick: handle_back_to_list,
                                    class: "flex-1 sm:flex-initial sm:min-w-[120px]",
                                    "Back to List"
                                }
                            }
                        } else {
                            div { class: "flex flex-col sm:flex-row justify-end gap-2",
                                Button {
                                    variant: ButtonVariant::Ghost,
                                    onclick: handle_back_to_list,
                                    class: "flex-1",
                                    "Back to List"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
