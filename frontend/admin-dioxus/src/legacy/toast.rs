//! Defines the [`Toast`] component and its sub-components, which provide a notification system for displaying temporary messages to users.

use crate::hooks::use_unique_id;

use super::portal_v2::{use_portal, PortalIn, PortalOut};
use dioxus::dioxus_core::DynamicNode;
use dioxus::logger::tracing;
use dioxus::prelude::*;
use dioxus_time::use_timeout;
use std::collections::VecDeque;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

impl ToastType {
    fn as_str(&self) -> &'static str {
        match self {
            ToastType::Success => "success",
            ToastType::Error => "error",
            ToastType::Warning => "warning",
            ToastType::Info => "info",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct ToastItem {
    id: usize,
    title: String,
    description: Option<String>,
    toast_type: ToastType,
    duration: Option<Duration>,
    permanent: bool,
    closing: bool,
}

type AddToastCallback = Callback<(String, Option<String>, ToastType, Option<Duration>, bool)>;

#[derive(Clone)]
struct ToastCtx {
    #[allow(dead_code)]
    toasts: Signal<VecDeque<ToastItem>>,
    add_toast: AddToastCallback,
    close_toast: Callback<usize>,
    remove_toast: Callback<usize>,
    focus_region: Callback,
}

#[derive(Props, Clone, PartialEq)]
pub struct ToastProviderProps {
    #[props(default = ReadSignal::new(Signal::new(Some(Duration::from_secs(3)))))]
    pub default_duration: ReadSignal<Option<Duration>>,
    #[props(default = ReadSignal::new(Signal::new(10)))]
    pub max_toasts: ReadSignal<usize>,
    #[props(default = Callback::new(|props: ToastPropsWithOwner| rsx! { {DynamicNode::Component(props.into_vcomponent(Toast))} }))]
    pub render_toast: Callback<ToastPropsWithOwner, Element>,
    children: Element,
}

#[component]
pub fn ToastProvider(props: ToastProviderProps) -> Element {
    let mut toasts = use_signal(VecDeque::new);
    let portal = use_portal();

    let remove_toast = use_callback(move |id: usize| {
        tracing::info!("[toast] remove_toast called: id={}", id);
        let mut toasts_vec = toasts.write();
        if let Some(pos) = toasts_vec.iter().position(|t: &ToastItem| t.id == id) {
            toasts_vec.remove(pos);
            tracing::info!("[toast] removed: id={}", id);
        } else {
            tracing::info!("[toast] not found (already removed?): id={}", id);
        }
    });

    // Start animated close: mark as closing and remove after animation duration
    let close_toast = use_callback(move |id: usize| {
        tracing::info!("[toast] close_toast start: id={}", id);
        {
            let mut toasts_vec = toasts.write();
            if let Some(pos) = toasts_vec.iter().position(|t: &ToastItem| t.id == id) {
                if !toasts_vec[pos].closing {
                    toasts_vec[pos].closing = true;
                }
            }
        }
        let remove_toast = remove_toast.clone();
        spawn(async move {
            gloo_timers::future::TimeoutFuture::new(200).await;
            // dioxus_time::sleep(Duration::from_millis(200)).await;
            remove_toast.call(id);
        });
    });

    let add_toast = use_callback(
        move |(title, description, toast_type, duration, permanent): (
            String,
            Option<String>,
            ToastType,
            Option<Duration>,
            bool,
        )| {
            use std::sync::atomic::{AtomicUsize, Ordering};
            static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
            let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
            // Determine effective default (fallback to 3s if provider default is None)
            let provided_default = (props.default_duration)();
            let effective_default = provided_default.or(Some(Duration::from_secs(3)));
            let duration = if permanent {
                None
            } else {
                duration.or(effective_default)
            };
            tracing::info!(
                "[toast] add_toast: id={}, permanent={}, provided_default={:?}, effective_default={:?}, effective_duration={:?}",
                id,
                permanent,
                provided_default,
                effective_default,
                duration
            );
            let toast = ToastItem {
                id,
                title,
                description,
                toast_type,
                duration,
                permanent,
                closing: false,
            };

            let mut toasts_vec = toasts.write();
            toasts_vec.push_back(toast.clone());
            let max = (props.max_toasts)();
            while toasts_vec.len() > max {
                if let Some(pos) = toasts_vec.iter().position(|t| !t.permanent) {
                    toasts_vec.remove(pos);
                } else {
                    toasts_vec.pop_front();
                }
            }

            // Provider-level auto-dismiss: schedule animated close if a duration exists
            if let Some(d) = duration {
                tracing::info!(
                    "[toast] provider timer scheduled: id={}, duration={:?}",
                    id,
                    d
                );
                let close = close_toast.clone();
                let remove_id = id;
                spawn(async move {
                    gloo_timers::future::TimeoutFuture::new(d.as_millis() as u32).await;
                    // dioxus_time::sleep(d).await;
                    tracing::info!("[toast] provider timer fired: id={}", remove_id);
                    close.call(remove_id);
                });
            }
        },
    );
    let toast_list = use_memo(move || {
        let toasts_vec = toasts.read();
        toasts_vec.iter().cloned().collect::<Vec<_>>()
    });
    let length = toast_list.len();

    let mut region_ref: Signal<Option<std::rc::Rc<MountedData>>> = use_signal(|| None);

    let focus_region = use_callback(move |_| {
        let Some(region_ref) = region_ref() else {
            return;
        };
        spawn(async move {
            _ = region_ref.set_focus(true).await;
        });
    });

    use_effect(move || {
        let mut eval = dioxus::document::eval(
            "document.addEventListener('keydown', (event) => { if (event.key === 'F6') { dioxus.send(true) } });",
        );
        spawn(async move {
            while let Ok(true) = eval.recv().await {
                focus_region(())
            }
        });
    });

    let ctx = use_context_provider(|| ToastCtx {
        toasts,
        add_toast,
        close_toast,
        remove_toast,
        focus_region,
    });

    rsx! {
        {props.children}
        PortalIn { portal,
            div {
                role: "region",
                aria_label: "{length} notifications",
                tabindex: "-1",
                class: "toast-container group/toast fixed z-[9999] right-5 bottom-5 max-w-[350px]",
                style: "--toast-count: {length}",
                onmounted: move |e| {
                    region_ref.set(Some(e.data()));
                },

                ol {
                    class: "toast-list flex flex-col-reverse p-0 m-0 gap-3 list-none",
                    for (index, toast) in toast_list.read().iter().rev().enumerate() {
                        li {
                            key: "{toast.id}",
                            class: "toast-item flex",
                            {
                                props.render_toast.call(ToastProps::builder().id(toast.id)
                                    .index(index)
                                    .title(toast.title.clone())
                                    .description(toast.description.clone())
                                    .toast_type(toast.toast_type)
                                    .permanent(toast.permanent)
                                    .closing(toast.closing)
                                    .on_close({
                                        let toast_id = toast.id;
                                        let close_toast = ctx.close_toast;
                                        move |_| {
                                            close_toast.call(toast_id);
                                        }
                                    })
                                    .duration({
                                        let provided_default = (props.default_duration)();
                                        let effective_default = provided_default.or(Some(Duration::from_secs(3)));
                                        if toast.permanent { None } else { toast.duration.or(effective_default) }
                                    })
                                    .attributes(vec![])
                                    .build()
                                )
                            }
                        }
                    }
                }
            }
        }
        PortalOut { portal }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ToastProps {
    pub id: usize,
    pub index: usize,
    pub title: String,
    pub description: Option<String>,
    pub toast_type: ToastType,
    pub on_close: Callback<MouseEvent>,
    #[props(default = false)]
    pub permanent: bool,
    pub duration: Option<Duration>,
    #[props(default = false)]
    pub closing: bool,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
}

#[component]
pub fn Toast(props: ToastProps) -> Element {
    let toast_id = use_unique_id();
    let id = use_memo(move || format!("toast-{toast_id}"));
    let label_id = format!("{id}-label");
    let description_id = props
        .description
        .as_ref()
        .map(|_| format!("{id}-description"));
    let ctx = use_context::<ToastCtx>();
    let effective_duration = if props.permanent {
        None
    } else {
        props.duration
    };
    let toast_id_for_timeout = props.id;
    let close_toast_for_timeout = ctx.close_toast;
    let timeout = use_timeout(
        effective_duration.unwrap_or(Duration::from_millis(1)),
        move |()| {
            tracing::info!("[toast] component timer fired: id={}", toast_id_for_timeout);
            close_toast_for_timeout.call(toast_id_for_timeout);
        },
    );
    let mut started = use_signal(|| false);
    use_effect(move || {
        if effective_duration.is_some() && !props.closing && !started() {
            tracing::info!(
                "[toast] component timer start: id={}, duration={:?}",
                toast_id_for_timeout,
                effective_duration
            );
            started.set(true);
            timeout.action(());
        }
    });

    rsx! {
        div {
            id,
            role: "alertdialog",
            aria_labelledby: "{label_id}",
            aria_describedby: description_id,
            aria_modal: "false",
            tabindex: "0",

            class: "toast flex overflow-hidden w-72 h-16 box-border items-center justify-between px-4 py-3 rounded-md border border-border z-[calc(var(--toast-count)-var(--toast-index))] -mt-16 shadow-[0_4px_12px_rgba(0,0,0,0.15)] transition-[transform,margin,opacity] duration-200 ease-out transform-gpu opacity-[calc(1-var(--toast-hidden))] scale-x-[calc(1-0.05*var(--toast-index))] scale-y-[calc(1-0.02*var(--toast-index))] group-hover/toast:animate-none group-focus-within/toast:animate-none dark:brightness-[calc(0.5+0.5*(1-((var(--toast-index)+1)/4)))] data-[type=success]:bg-[var(--primary-success-color)] data-[type=success]:text-[var(--secondary-success-color)] data-[type=error]:bg-[var(--primary-error-color)] data-[type=error]:text-[var(--contrast-error-color)] data-[type=warning]:bg-[var(--primary-warning-color)] data-[type=warning]:text-[var(--secondary-warning-color)] data-[type=info]:bg-[var(--primary-info-color)] data-[type=info]:text-[var(--secondary-info-color)] data-[closing=true]:opacity-0 data-[closing=true]:translate-y-2",
            "data-type": props.toast_type.as_str(),
            "data-permanent": props.permanent,
            "data-closing": props.closing.then_some("true"),
            "data-toast-even": (props.index % 2 == 0).then_some("true"),
            "data-toast-odd": (props.index % 2 == 1).then_some("true"),
            "data-top": (props.index == 0).then_some("true"),
            style: "--toast-index: {props.index}; --toast-hidden: calc(min(max(0, var(--toast-index) - 2), 1))",
            ..props.attributes,

            div { class: "toast-content flex-1 mr-2 transition-[filter] duration-200",
                role: "alert",
                aria_atomic: "true",

                div {
                    id: label_id,
                    class: "toast-title mb-1 text-foreground font-semibold",
                    {props.title.clone()}
                }

                if let Some(description) = &props.description {
                    div {
                        id: description_id.clone(),
                        class: "toast-description text-muted-foreground text-sm",
                        {description.clone()}
                    }
                }
            }

            button {
                class: "toast-close self-start p-0 m-0 border-0 bg-transparent text-[18px] leading-none cursor-pointer text-muted-foreground hover:text-foreground",
                aria_label: "close",
                onclick: move |e| {
                    ctx.focus_region.call(());
                    props.on_close.call(e);
                },
                "×"
            }
        }
    }
}

#[derive(Clone, Default)]
pub struct ToastOptions {
    description: Option<String>,
    duration: Option<Duration>,
    permanent: bool,
}

impl ToastOptions {
    pub fn new() -> Self {
        Self {
            description: None,
            duration: None,
            permanent: false,
        }
    }

    pub fn description(mut self, description: impl ToString) -> Self {
        self.description = Some(description.to_string());
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    pub fn permanent(mut self, permanent: bool) -> Self {
        self.permanent = permanent;
        self
    }
}

#[derive(Clone, Copy)]
pub struct Toasts {
    add_toast: AddToastCallback,
    #[allow(dead_code)]
    remove_toast: Callback<usize>,
}

impl Toasts {
    pub fn show(&self, title: String, toast_type: ToastType, options: ToastOptions) {
        self.add_toast.call((
            title,
            options.description,
            toast_type,
            if options.permanent {
                None
            } else {
                options.duration
            },
            options.permanent,
        ));
    }

    pub fn success(&self, title: String, options: ToastOptions) {
        self.show(title, ToastType::Success, options);
    }
    pub fn error(&self, title: String, options: ToastOptions) {
        self.show(title, ToastType::Error, options);
    }
    pub fn warning(&self, title: String, options: ToastOptions) {
        self.show(title, ToastType::Warning, options);
    }
    pub fn info(&self, title: String, options: ToastOptions) {
        self.show(title, ToastType::Info, options);
    }
}

pub fn use_toast() -> Toasts {
    use_hook(consume_toast)
}

pub fn consume_toast() -> Toasts {
    let ctx = consume_context::<ToastCtx>();
    let add_toast = ctx.add_toast;
    let remove_toast = ctx.remove_toast;

    Toasts {
        add_toast,
        remove_toast,
    }
}
