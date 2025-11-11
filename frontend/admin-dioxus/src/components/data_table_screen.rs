use dioxus::prelude::*;

use crate::components::{
    ErrorDetails, ErrorDetailsVariant, ListToolbar, ListToolbarProps, LoadingOverlay, PageHeader,
    PageHeaderProps, Pagination,
};
use crate::store::{AppError, PaginatedList, StateFrame};
use crate::ui::shadcn::{Button, ButtonVariant};
use hmziq_dioxus_free_icons::{icons::ld_icons::LdArrowUpDown, Icon};

#[derive(Debug, Clone, PartialEq)]
pub struct HeaderColumn {
    pub label: String,
    pub sortable: bool,
    pub class: String,
    pub field: Option<String>,
}

impl HeaderColumn {
    pub fn new(label: &str, sortable: bool, class: &str, field: Option<&str>) -> Self {
        Self {
            label: label.to_string(),
            sortable,
            class: class.to_string(),
            field: field.map(|s| s.to_string()),
        }
    }
}

#[derive(Props, PartialEq, Clone)]
pub struct DataTableScreenProps<T: Clone + PartialEq + 'static> {
    /// State frame carrying paginated data for the table (clone of state)
    pub frame: StateFrame<PaginatedList<T>>,

    /// Page header configuration (spread as `PageHeader { ..props }`)
    #[props(optional)]
    pub header: Option<PageHeaderProps>,

    /// Optional custom title for error details (defaults to a generic label)
    #[props(default)]
    pub error_title: Option<String>,

    /// Variant for the error details component
    #[props(default = ErrorDetailsVariant::Collapsed)]
    pub error_variant: ErrorDetailsVariant,

    /// Label for retry button rendered under the error details
    #[props(default)]
    pub error_retry_label: Option<String>,

    /// Retry handler for the button
    #[props(default)]
    pub on_error_retry: Option<EventHandler<()>>,

    /// Fallback message when the state frame does not include an AppError
    #[props(default = "Failed to load data. Please try again.".to_string())]
    pub error_fallback_message: String,

    /// Toolbar configuration (spread as `ListToolbar { ..props }`)
    #[props(optional)]
    pub toolbar: Option<ListToolbarProps>,

    /// Optional header columns configuration for automatic thead generation
    #[props(optional)]
    pub headers: Option<Vec<HeaderColumn>>,

    /// Current sort field for header highlighting
    #[props(optional)]
    pub current_sort_field: Option<String>,

    /// Callback for header sort click
    #[props(optional)]
    pub on_sort: Option<EventHandler<String>>,

    /// Custom table body markup to render inside tbody
    pub children: Element,

    /// Pagination handlers
    pub on_prev: EventHandler<()>,
    pub on_next: EventHandler<()>,

    /// Toggle pagination / overlay if needed
    #[props(default = true)]
    pub show_pagination: bool,
    #[props(default = true)]
    pub show_loading_overlay: bool,

    /// Optional center overlay content rendered absolutely centered over the table container
    #[props(optional)]
    pub center_overlay: Option<Element>,

    /// Optional content rendered between the toolbar and the table (e.g., bulk actions bar)
    #[props(optional)]
    pub below_toolbar: Option<Element>,
}

#[component]
pub fn DataTableScreen<T: Clone + PartialEq + 'static>(props: DataTableScreenProps<T>) -> Element {
    let list = props.frame.clone();
    let list_loading = list.is_loading();
    let list_failed = list.is_failed();

    let has_data = list
        .data
        .as_ref()
        .map(|p| !p.data.is_empty())
        .unwrap_or(false);

    rsx! {
        div { class: "min-h-screen bg-transparent",
            // Page header (optional)
            if let Some(header_props) = props.header.clone() {
                PageHeader { ..header_props }
            }

            // Error banner (only when failed)
            if list_failed {
                {
                    let error = list.error.clone().or_else(|| {
                        Some(AppError::Other {
                            message: props.error_fallback_message.clone(),
                        })
                    });

                    rsx! {
                        div { class: "container mx-auto px-4 pt-4",
                            ErrorDetails {
                                error,
                                variant: props.error_variant,
                                title: props.error_title.clone(),
                                class: Some("w-full".to_string()),
                            }
                            if let (Some(label), Some(on_retry)) = (props.error_retry_label.clone(), props.on_error_retry.clone()) {
                                div { class: "mt-4",
                                    Button {
                                        variant: ButtonVariant::Outline,
                                        onclick: move |_| { on_retry.call(()); },
                                        "{label}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Main content
            div { class: "container mx-auto px-4 py-8 md:py-12",
                // Toolbar (optional)
                if let Some(toolbar_props) = props.toolbar.clone() {
                    ListToolbar { ..toolbar_props }
                }

                // Below-toolbar slot (optional)
                if let Some(below) = props.below_toolbar.clone() {
                    div { class: "mt-6 md:mt-8 mb-6 md:mb-8",
                        {below}
                    }
                }

                div { class: "bg-transparent border border-zinc-200 dark:border-zinc-800 rounded-lg mt-10 md:mt-12",
                    div { class: "relative",
                        div { class: "overflow-x-auto",
                            table { class: "w-full min-w-[720px] text-xs md:text-sm",
                                // Auto-generated thead if headers provided
                                if let Some(headers) = &props.headers {
                                    thead { class: "bg-transparent",
                                        tr { class: "border-b border-zinc-200 dark:border-zinc-800 hover:bg-transparent",
                                            {headers.iter().map(|header| {
                                                let is_current_sort = props.current_sort_field.as_ref()
                                                    .and_then(|field| header.field.as_ref().map(|f| f == field))
                                                    .unwrap_or(false);

                                                rsx! {
                                                    th { class: "{header.class}",
                                                        if header.sortable {
                                                        Button {
                                                            variant: ButtonVariant::Ghost,
                                                            class: "h-8 bg-transparent hover:bg-muted/50 -ml-3 text-left justify-start font-medium p-2 text-xs md:text-sm",
                                                                onclick: {
                                                                    let field = header.field.clone().unwrap_or_default();
                                                                    let on_sort = props.on_sort.clone();
                                                                    move |_| {
                                                                        if let Some(handler) = &on_sort {
                                                                            handler.call(field.clone());
                                                                        }
                                                                    }
                                                                },
                                                                "{header.label}"
                                                                if is_current_sort {
                                                                    div { class: "ml-2 h-4 w-4", Icon { icon: LdArrowUpDown {} } }
                                                                }
                                                            }
                                                        } else {
                                                            "{header.label}"
                                                        }
                                                    }
                                                }
                                            })}
                                        }
                                    }
                                }

                                // Caller-provided tbody content
                                tbody {
                                    { props.children }
                                }
                            }
                        }

                        // Center overlay (e.g., bulk actions when rows are selected)
                        if let Some(center_node) = props.center_overlay.clone() {
                            div { class: "absolute inset-0 z-10 flex items-center justify-center pointer-events-none",
                                // re-enable pointer events only for the action content
                                div { class: "pointer-events-auto",
                                    {center_node}
                                }
                            }
                        }


                        // Pagination
                        if props.show_pagination {
                            Pagination::<T> {
                                page: list.data.clone(),
                                disabled: list_loading,
                                on_prev: move |_| { props.on_prev.call(()); },
                                on_next: move |_| { props.on_next.call(()); },
                            }
                        }

                        // Loading overlay when we have data
                        if props.show_loading_overlay {
                            LoadingOverlay { visible: list_loading && has_data }
                        }
                    }
                }
            }
        }
    }
}
