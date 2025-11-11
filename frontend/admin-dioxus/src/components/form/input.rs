use crate::hooks::{OxForm, OxFormModel};
use dioxus::prelude::*;

const BASE_INPUT_CLASS: &str = "w-full rounded-md border border-border/70 bg-transparent px-4 py-2.5 text-foreground placeholder:text-muted-foreground transition-colors duration-200 focus:border-ring focus:ring-2 focus:ring-ring/40 disabled:cursor-not-allowed disabled:opacity-60";

fn compose_input_class(extra: &Option<String>) -> String {
    let mut classes = vec![BASE_INPUT_CLASS.to_string()];
    if let Some(extra) = extra {
        classes.push(extra.clone());
    }
    classes.join(" ")
}

#[derive(Props, PartialEq, Clone)]
pub struct AppInputProps<T: OxFormModel + 'static> {
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
    #[props(default = String::from("text"))]
    r#type: String,
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
pub fn AppInput<T>(props: AppInputProps<T>) -> Element
where
    T: OxFormModel + 'static,
{
    let mut form = props.form.clone();
    let name = props.name.clone();

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
            input {
                id: props.id.clone(),
                disabled: props.disabled,
                readonly: props.readonly,
                r#type: props.r#type.clone(),
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
            if let Some(error) = &field.error {
                p { class: "my-2 text-sm text-destructive transition-colors duration-200",
                    {error.clone()}
                }
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct SimpleInputProps {
    pub value: String,
    #[props(default = None)]
    pub placeholder: Option<String>,
    #[props(default = None)]
    pub id: Option<String>,
    #[props(default = None)]
    pub class: Option<String>,
    #[props(default = String::from("text"))]
    pub r#type: String,
    #[props(default = None)]
    pub oninput: Option<EventHandler<String>>,
    #[props(default = None)]
    pub onchange: Option<EventHandler<String>>,
    #[props(default = false)]
    pub disabled: bool,
    #[props(default = false)]
    pub readonly: bool,
}

#[component]
pub fn SimpleInput(props: SimpleInputProps) -> Element {
    let input_class = compose_input_class(&props.class);
    let oninput_handler = props.oninput.clone();
    let onchange_handler = props.onchange.clone();

    rsx! {
        input {
            id: props.id.clone(),
            r#type: props.r#type.clone(),
            class: "{input_class}",
            value: props.value.clone(),
            placeholder: props.placeholder.clone(),
            disabled: props.disabled,
            readonly: props.readonly,
            oninput: move |event| {
                if let Some(handler) = &oninput_handler {
                    handler.call(event.value());
                }
            },
            onchange: move |event| {
                if let Some(handler) = &onchange_handler {
                    handler.call(event.value());
                }
            },
        }
    }
}
