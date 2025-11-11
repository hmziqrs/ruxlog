use dioxus::prelude::*;

use super::form::{use_tag_form, TagForm};
use crate::components::{AppInput, ColorPicker, TagBadge, TagSize};
use crate::hooks::OxForm;
use crate::router::Route;
use crate::store::Tag;
use crate::ui::custom::AppPortal;
use crate::ui::shadcn::{Button, ButtonSize, ButtonVariant, Checkbox};
use crate::utils::colors::get_contrast_yiq;

#[derive(Props, PartialEq, Clone)]
pub struct TagFormContainerProps {
    #[props(default)]
    pub initial: Option<TagForm>,
    pub on_submit: EventHandler<TagForm>,
    #[props(default)]
    pub title: Option<String>,
    #[props(default)]
    pub submit_label: Option<String>,
}

#[component]
pub fn TagFormContainer(props: TagFormContainerProps) -> Element {
    let nav = use_navigator();
    let initial_tag_form = props.initial.clone().unwrap_or_else(TagForm::new);
    let reset_template = initial_tag_form.clone();
    let tag_form_hook = use_tag_form(initial_tag_form);
    let mut form = tag_form_hook.form;
    let mut reset_dialog_open = use_signal(|| false);
    let is_form_dirty = form.read().is_dirty();

    rsx! {
        div {
            if let Some(t) = props.title.clone() { h1 { class: "sr-only", {t} } }

            div { class: "grid grid-cols-1 gap-10 lg:grid-cols-3",
                div { class: "lg:col-span-2 space-y-8",
                    div { class: "rounded-xl border border-border/70 bg-transparent",
                        div { class: "px-6 py-6",
                            h2 { class: "text-lg font-semibold", "Tag details" }
                            p { class: "text-sm text-muted-foreground", "Basic information and metadata for your tag." }
                        }
                        div { class: "px-6 py-6 space-y-6",
                            AppInput { name: "name", form, label: "Name", placeholder: "e.g. Product Updates" }

                            div { class: "h-px bg-border/60" }

                            div { class: "space-y-3",
                                div { class: "flex items-center justify-between",
                                    label { class: "block text-sm font-medium text-foreground", "Slug" }
                                    Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm,
                                        onclick: move |_| {
                                            let name_value = form.peek().get_field("name").unwrap().value.clone();
                                            if !name_value.is_empty() {
                                                let sanitized = TagForm::sanitize_slug(&name_value);
                                                form.write().update_field("slug", sanitized);
                                            }
                                        },
                                        "Generate from name"
                                    }
                                }
                                AppInput { form, name: "slug", r#type: "text", placeholder: "product-updates" }
                                div { class: "flex items-center gap-2",
                                    span { class: "text-xs text-muted-foreground", "URL preview:" }
                                    code { class: "rounded border border-border/60 bg-transparent px-1.5 py-0.5 text-xs text-muted-foreground",
                                        {
                                            let slug = form.read().data.slug.clone();
                                            let safe = if slug.trim().is_empty() { "your-tag-slug".to_string() } else { slug };
                                            format!("/tags/{}", safe)
                                        }
                                    }
                                }
                            }

                            div { class: "h-px bg-border/60" }

                            div { class: "space-y-3",
                                label { class: "block text-sm font-medium text-foreground", "Description" }
                                textarea {
                                    class: "w-full h-32 resize-none rounded-md border border-border/70 bg-transparent px-4 py-3 text-sm text-foreground placeholder:text-muted-foreground transition-colors duration-200 focus:border-ring focus:ring-2 focus:ring-ring/40",
                                    placeholder: "Briefly describe what posts belong in this tag.",
                                    value: form.read().data.description.clone(),
                                    oninput: move |event| { form.write().update_field("description", event.value()); }
                                }
                            }
                        }
                    }

                    div { class: "flex items-start gap-3 rounded-lg border border-border/60 bg-transparent p-5",
                        div { class: "mt-0.5 h-4 w-4 rounded-full border border-border/40" }
                        div { class: "space-y-1",
                            p { class: "text-sm font-medium text-foreground", "Design tip" }
                            p { class: "text-sm text-muted-foreground", "Use contrasting colors to ensure the tag remains legible in both light and dark themes." }
                        }
                    }
                }

                div { class: "space-y-8 lg:sticky lg:top-28 h-fit",
                    div { class: "rounded-xl border border-border/70 bg-transparent",
                        div { class: "px-6 pt-6",
                            h2 { class: "text-lg font-semibold", "Appearance" }
                            p { class: "text-sm text-muted-foreground", "Choose a color and preview the tag." }
                        }
                        div { class: "px-6 py-6 space-y-6",
                            div { class: "space-y-3",
                                label { class: "block text-sm font-medium text-foreground", "Tag color" }
                                ColorPicker { value: form.read().data.color.clone(), onchange: move |val| { form.write().update_field("color", val); } }
                            }
                            div { class: "space-y-3",
                                div { class: "flex items-center justify-between",
                                    div { class: "space-y-0.5",
                                        label { class: "block text-sm font-medium text-foreground", "Text color" }
                                        p { class: "text-xs text-muted-foreground", "Enable to choose a custom text color. Otherwise it auto-adjusts for readability." }
                                    }
                                    Checkbox { checked: form.read().data.custom_text_color, onchange: move |checked: bool| { form.write().update_field("custom_text_color", checked.to_string()); } }
                                }
                                if form.read().data.custom_text_color {
                                    ColorPicker { value: form.read().data.text_color.clone(), onchange: move |val| { form.write().update_field("text_color", val); } }
                                }
                            }
                            {
                                let data = form.read().data.clone();
                                let color = data.color.clone();
                                let text_color = if data.custom_text_color && !data.text_color.trim().is_empty() { data.text_color.clone() } else { get_contrast_yiq(&color).to_string() };
                                let preview_tag = Tag {
                                    name: if data.name.is_empty() { "Tag preview".to_string() } else { data.name.clone() },
                                    slug: data.slug.clone(),
                                    description: if data.description.trim().is_empty() { None } else { Some(data.description.clone()) },
                                    color: color.clone(),
                                    text_color: text_color.clone(),
                                    is_active: data.active,
                                    ..Tag::default()
                                };
                                rsx! {
                                    div { class: "space-y-3",
                                        label { class: "block text-sm font-medium text-foreground", "Preview" }
                                        div { class: "flex items-center gap-3",
                                            TagBadge { tag: preview_tag.clone(), size: TagSize::Md }
                                            code { class: "text-xs rounded border border-border/60 bg-transparent px-1.5 py-0.5 text-muted-foreground", {color} }
                                        }
                                        p { class: "text-xs text-muted-foreground", if data.custom_text_color { "Using custom text color." } else { "Text color auto-adjusts for readability." } }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "rounded-xl border border-border/70 bg-transparent",
                        div { class: "px-6 pt-6",
                            h2 { class: "text-lg font-semibold", "Visibility" }
                            p { class: "text-sm text-muted-foreground", "Control whether this tag is available publicly." }
                        }
                        div { class: "px-6 py-6 space-y-6",
                            div { class: "flex items-center justify-between",
                                div { class: "space-y-0.5",
                                    label { class: "block text-sm font-medium text-foreground", "Active" }
                                    p { class: "text-xs text-muted-foreground", if form.read().data.active { "This tag will be visible across your site." } else { "This tag will be hidden and unavailable for selection." } }
                                }
                                Checkbox { class: None, checked: form.read().data.active, onchange: move |checked: bool| { form.write().update_field("active", checked.to_string()); } }
                            }
                        }
                    }

                    div { class: "flex gap-3 pt-4",
                        Button { class: "flex-1 w-auto", variant: ButtonVariant::Outline,
                            onclick: move |_| {
                                if form.peek().is_dirty() {
                                    reset_dialog_open.set(true);
                                } else {
                                    nav.push(Route::TagsListScreen {});
                                }
                            },
                            {if is_form_dirty { "Reset" } else { "Cancel" }}
                        }
                        Button { class: "flex-1 w-auto",
                            onclick: move |_| {
                                let submit = props.on_submit.clone();
                                form.write().on_submit(move |val| { submit.call(val); });
                            },
                            {props.submit_label.clone().unwrap_or_else(|| "Save Tag".to_string())}
                        }
                    }
                }
            }
        }
        if reset_dialog_open() {
            AppPortal {
                class: "bg-black/20 backdrop-blur-sm flex items-center justify-center px-4",
                div { class: "w-full max-w-md rounded-lg border border-border/60 bg-background p-6 shadow-lg",
                    div { class: "space-y-2",
                        h2 { class: "text-lg font-semibold", "Reset form?" }
                        p { class: "text-sm text-muted-foreground", "All changes will be cleared and the form will return to its default state." }
                    }
                    div { class: "mt-6 flex justify-end gap-2",
                        Button { variant: ButtonVariant::Outline,
                            onclick: move |_| {
                                reset_dialog_open.set(false);
                            },
                            "Cancel"
                        }
                        Button { variant: ButtonVariant::Destructive,
                            onclick: move |_| {
                                form.set(OxForm::new(reset_template.clone()));
                                reset_dialog_open.set(false);
                            },
                            "Reset form"
                        }
                    }
                }
            }
        }
    }
}
