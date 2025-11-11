use crate::store::AppError;
use crate::ui::shadcn::{Button, ButtonSize, ButtonVariant, Dialog, DialogContent, DialogTrigger};
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::{
    icons::ld_icons::{LdChevronDown, LdChevronUp, LdInfo, LdTriangleAlert},
    Icon,
};

#[derive(PartialEq, Clone, Copy)]
pub enum ErrorDetailsVariant {
    Minimum,
    Collapsed,
    Expanded,
}

#[derive(Props, PartialEq, Clone)]
pub struct ErrorDetailsProps {
    pub error: Option<AppError>,
    #[props(default = ErrorDetailsVariant::Collapsed)]
    pub variant: ErrorDetailsVariant,
    #[props(default)]
    pub title: Option<String>,
    #[props(default)]
    pub class: Option<String>,
}

#[component]
pub fn ErrorDetails(props: ErrorDetailsProps) -> Element {
    let Some(error) = props.error.clone() else {
        return rsx! {};
    };

    match props.variant {
        ErrorDetailsVariant::Minimum => rsx! {
            ErrorDetailsMinimum {
                error,
                title: props.title.clone(),
                class: props.class.clone(),
            }
        },
        ErrorDetailsVariant::Collapsed => rsx! {
            ErrorDetailsCollapsed {
                error,
                title: props.title.clone(),
                class: props.class.clone(),
            }
        },
        ErrorDetailsVariant::Expanded => rsx! {
            ErrorDetailsExpanded {
                error,
                title: props.title.clone(),
                class: props.class.clone(),
            }
        },
    }
}

#[derive(Props, PartialEq, Clone)]
struct ErrorDetailsVariantProps {
    error: AppError,
    #[props(default)]
    title: Option<String>,
    #[props(default)]
    class: Option<String>,
}

#[component]
fn ErrorDetailsMinimum(props: ErrorDetailsVariantProps) -> Element {
    let message = props.error.message();
    let title = props
        .title
        .clone()
        .unwrap_or_else(|| default_title(&props.error).to_string());
    let class = props.class.clone().unwrap_or_default();

    rsx! {
        Dialog {
            div { class: format!("inline-flex items-center gap-2 rounded-lg border border-border/70 bg-transparent px-3 py-2 {}", class),
                span { class: "flex-1 text-sm text-foreground", {message.clone()} }
                DialogTrigger {
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Sm,
                        class: "size-9 rounded-full border border-border/60 px-0",
                        aria_label: Some("View error details".to_string()),
                        Icon { icon: LdInfo {} }
                    }
                }
            }
            DialogContent {
                ErrorDetailsCard {
                    error: props.error.clone(),
                    title: Some(title),
                    class: Some("border-border/60 shadow-none".to_string()),
                }
            }
        }
    }
}

#[component]
fn ErrorDetailsCollapsed(props: ErrorDetailsVariantProps) -> Element {
    let message = props.error.message();
    let title = props
        .title
        .clone()
        .unwrap_or_else(|| default_title(&props.error).to_string());
    let hint = hint_for_error(&props.error);
    let class = props.class.clone().unwrap_or_default();
    let mut show_details = use_signal(|| false);

    rsx! {
        div { class: format!("space-y-4 {}", class),
            div { class: "flex w-full items-start gap-3 rounded-lg border border-border/70 bg-transparent p-4",
                div { class: "flex-1 space-y-1",
                    p { class: "text-sm font-semibold text-foreground", {title.clone()} }
                    p { class: "text-sm text-muted-foreground", {message.clone()} }
                    if let Some(h) = hint {
                        p { class: "text-xs text-muted-foreground", {h} }
                    }
                }
                Button {
                    variant: ButtonVariant::Ghost,
                    size: ButtonSize::Sm,
                    class: "size-9 rounded-full border border-border/60 px-0",
                    aria_label: Some(
                        if show_details() {
                            "Collapse error details".to_string()
                        } else {
                            "Expand error details".to_string()
                        }
                    ),
                    onclick: move |_| {
                        show_details.set(!show_details());
                    },
                    if show_details() {
                        Icon { icon: LdChevronUp {} }
                    } else {
                        Icon { icon: LdChevronDown {} }
                    }
                }
            }
            if show_details() {
                ErrorDetailsCard {
                    error: props.error.clone(),
                    title: Some(title),
                    class: Some("border-border/60".to_string()),
                }
            }
        }
    }
}

#[component]
fn ErrorDetailsExpanded(props: ErrorDetailsVariantProps) -> Element {
    let class = props.class.clone().unwrap_or_default();
    rsx! {
        div { class: class,
            ErrorDetailsCard {
                error: props.error.clone(),
                title: props.title.clone(),
                class: Some("border-border/70".to_string()),
            }
        }
    }
}

#[derive(Props, PartialEq, Clone)]
struct ErrorDetailsCardProps {
    error: AppError,
    #[props(default)]
    title: Option<String>,
    #[props(default)]
    class: Option<String>,
}

#[component]
fn ErrorDetailsCard(props: ErrorDetailsCardProps) -> Element {
    let title = props
        .title
        .clone()
        .unwrap_or_else(|| default_title(&props.error).to_string());
    let message = props.error.message();
    let rows = collect_detail_rows(&props.error);
    let icon_is_alert = !matches!(props.error, AppError::Other { .. });
    let mut container_classes = vec!["w-full rounded-lg bg-transparent".to_string()];
    if let Some(custom) = props.class.clone() {
        container_classes.push(custom);
    }

    rsx! {
        div { class: container_classes.join(" "),
            div { class: "space-y-4 p-5 sm:p-6",
                div { class: "flex items-center gap-2 text-base",
                    div { class: "text-destructive [&_svg]:size-5",
                        if icon_is_alert {
                            Icon { icon: LdTriangleAlert {} }
                        } else {
                            Icon { icon: LdInfo {} }
                        }
                    }
                    span { class: "text-base font-semibold", {title} }
                }
                p { class: "text-sm text-muted-foreground", {message} }
                div { class: "border-t border-border/60 pt-4",
                    {render_detail_rows(&rows)}
                }
            }
        }
    }
}

#[derive(Clone)]
struct DetailRow {
    label: &'static str,
    value: String,
    monospace: bool,
}

impl DetailRow {
    fn text(label: &'static str, value: impl Into<String>) -> Self {
        Self {
            label,
            value: value.into(),
            monospace: false,
        }
    }

    fn code(label: &'static str, value: impl Into<String>) -> Self {
        Self {
            label,
            value: value.into(),
            monospace: true,
        }
    }
}

fn collect_detail_rows(error: &AppError) -> Vec<DetailRow> {
    let mut rows = Vec::new();
    match error {
        AppError::Api(api) => {
            if let Some(code) = &api.r#type {
                rows.push(DetailRow::text("Error code", code.clone()));
            }
            rows.push(DetailRow::text("HTTP status", api.status.to_string()));
            if let Some(details) = &api.details {
                rows.push(DetailRow::text("Details", details.clone()));
            }
            if let Some(context) = &api.context {
                if let Ok(pretty) = serde_json::to_string_pretty::<serde_json::Value>(context) {
                    rows.push(DetailRow::code("Context", pretty));
                } else {
                    rows.push(DetailRow::code("Context", context.to_string()));
                }
            }
            if let Some(request_id) = &api.request_id {
                rows.push(DetailRow::text("Request ID", request_id.clone()));
            }
            if let Some(retry_after) = api.retry_after {
                rows.push(DetailRow::text("Retry After (s)", retry_after.to_string()));
            }
        }
        AppError::Transport(info) => {
            rows.push(DetailRow::text("Category", info.kind.label().to_string()));
            if let Some(message) = &info.message {
                rows.push(DetailRow::text("Details", message.clone()));
            }
            if let Some(hint) = info.kind.hint() {
                rows.push(DetailRow::text("Suggested action", hint.to_string()));
            }
        }
        AppError::Decode {
            label,
            error: decode_error,
            raw: raw_payload,
        } => {
            rows.push(DetailRow::text("Deserializer", label.clone()));
            rows.push(DetailRow::text("Details", decode_error.clone()));

            if let Some(raw) = raw_payload {
                let (value, truncated) = truncate(raw, 2_000);
                let label = if truncated {
                    "Raw payload (truncated)"
                } else {
                    "Raw payload"
                };
                rows.push(DetailRow::code(label, value));
            }
        }
        AppError::Other { message } => {
            rows.push(DetailRow::text("Details", message.clone()));
        }
    }

    rows
}

fn render_detail_rows(rows: &[DetailRow]) -> Element {
    if rows.is_empty() {
        rsx! {
            p { class: "text-sm text-muted-foreground", "No additional diagnostics available." }
        }
    } else {
        rsx! {
            div { class: "grid gap-3 sm:grid-cols-2 w-full",
                for (idx, row) in rows.iter().enumerate() {
                    div { key: "{idx}", class: "rounded-lg border border-border/60 bg-transparent p-3",
                        p { class: "text-xs font-semibold uppercase tracking-wide text-muted-foreground", {row.label} }
                        if row.monospace {
                            pre { class: "mt-2 whitespace-pre-wrap break-words font-mono text-xs text-foreground/90", {row.value.clone()} }
                        } else {
                            p { class: "mt-2 whitespace-pre-wrap text-sm text-foreground", {row.value.clone()} }
                        }
                    }
                }
            }
        }
    }
}

fn hint_for_error(error: &AppError) -> Option<&'static str> {
    match error {
        AppError::Transport(info) => info.kind.hint(),
        AppError::Decode { .. } => Some("Response payload no longer matches the expected schema."),
        _ => None,
    }
}

fn default_title(error: &AppError) -> &'static str {
    match error {
        AppError::Api(_) => "API error",
        AppError::Transport(_) => "Network issue",
        AppError::Decode { .. } => "Decode error",
        AppError::Other { .. } => "Application error",
    }
}

fn truncate(input: &str, limit: usize) -> (String, bool) {
    if input.chars().count() <= limit {
        (input.to_string(), false)
    } else {
        let truncated: String = input.chars().take(limit).collect();
        (format!("{truncated}…"), true)
    }
}
