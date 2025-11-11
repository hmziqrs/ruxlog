use crate::hooks::{OxForm, OxFormModel};
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::icons::ld_icons::{LdEye, LdEyeOff};
use hmziq_dioxus_free_icons::Icon;

const BASE_INPUT_CLASS: &str = "w-full rounded-md border border-border/70 bg-transparent px-4 py-2.5 pr-11 text-foreground placeholder:text-muted-foreground transition-colors duration-200 focus:border-ring focus:ring-2 focus:ring-ring/40 disabled:cursor-not-allowed disabled:opacity-60";

fn compose_input_class(extra: &Option<String>) -> String {
    let mut classes = vec![BASE_INPUT_CLASS.to_string()];
    if let Some(extra) = extra {
        classes.push(extra.clone());
    }
    classes.join(" ")
}

#[derive(Props, PartialEq, Clone)]
pub struct PasswordInputProps<T: OxFormModel + 'static> {
    form: Signal<OxForm<T>>,
    name: String,
    #[props(default = None)]
    label: Option<String>,
    #[props(default = None)]
    placeholder: Option<String>,
    #[props(default = None)]
    id: Option<String>,
    #[props(default = None)]
    class: Option<String>,
    #[props(default = None)]
    onchange: Option<EventHandler<String>>,
    #[props(default = None)]
    onblur: Option<EventHandler<()>>,
    #[props(default = None)]
    onfocus: Option<EventHandler<()>>,
    #[props(default = false)]
    disabled: bool,
    #[props(default = false)]
    readonly: bool,
}

#[component]
pub fn PasswordInput<T>(props: PasswordInputProps<T>) -> Element
where
    T: OxFormModel + 'static,
{
    let mut form = props.form.clone();
    let name = props.name.clone();
    let mut is_visible = use_signal(|| false);

    let field = {
        let read = form.read();
        read.get_field(&name).unwrap().clone()
    };

    let name_on = name.clone();
    let name_ob = name.clone();
    let name_of = name.clone();

    let onchange_handler = props.onchange.clone();
    let onblur_handler = props.onblur.clone();
    let onfocus_handler = props.onfocus.clone();

    let input_class = compose_input_class(&props.class);

    rsx! {
        div {
            if let Some(label) = &props.label {
                label { class: "mb-2 block text-sm font-medium text-foreground transition-colors duration-200",
                    {label.clone()}
                }
            }
            div { class: "relative",
                input {
                    id: props.id.clone(),
                    disabled: props.disabled,
                    readonly: props.readonly,
                    r#type: if is_visible() { "text" } else { "password" },
                    class: "{input_class}",
                    value: field.value.clone(),
                    placeholder: props.placeholder.clone(),
                    onchange: move |event| {
                        let value = event.value();
                        if let Some(handler) = &onchange_handler {
                            handler.call(value.clone());
                        }
                        form.write().update_field(&name_on, value);
                    },
                    onblur: move |_| {
                        if let Some(handler) = &onblur_handler {
                            handler.call(());
                        }
                        form.write().blur_field(&name_ob);
                    },
                    onfocus: move |_| {
                        if let Some(handler) = &onfocus_handler {
                            handler.call(());
                        }
                        form.write().focus_field(&name_of);
                    },
                }
                button {
                    r#type: "button",
                    class: "absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors duration-200 focus:outline-none",
                    onclick: move |_| {
                        is_visible.set(!is_visible());
                    },
                    div { class: "w-5 h-5",
                        if is_visible() {
                            Icon { icon: LdEyeOff {} }
                        } else {
                            Icon { icon: LdEye {} }
                        }
                    }
                }
            }
            if let Some(error) = &field.error {
                p { class: "my-2 text-sm text-destructive transition-colors duration-200",
                    {error.clone()}
                }
            }
        }
    }
}
