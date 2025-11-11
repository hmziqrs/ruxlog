use dioxus::prelude::*;

use crate::router::Route;
use crate::ui::shadcn::{
    Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator,
};

#[derive(Props, PartialEq, Clone)]
pub struct PageHeaderProps {
    pub title: String,
    pub description: String,
    #[props(optional)]
    pub actions: Option<Element>, // right side actions (e.g., Create button)
    #[props(optional)]
    pub class: Option<String>, // optional class to tweak paddings
    #[props(default = false)]
    pub embedded: bool, // when true, render without outer wrapper and container
}

#[component]
pub fn PageHeader(props: PageHeaderProps) -> Element {
    let nav = use_navigator();
    let current_route = use_route::<Route>();

    // Derive breadcrumb segments: (text, optional link route)
    let segments: Vec<(String, Option<Route>)> = match current_route {
        Route::PostsAddScreen {} => vec![
            ("posts".to_string(), Some(Route::PostsListScreen {})),
            ("add".to_string(), None),
        ],
        Route::PostsEditScreen { id } => vec![
            ("posts".to_string(), Some(Route::PostsListScreen {})),
            (id.to_string(), Some(Route::PostsViewScreen { id })),
            ("edit".to_string(), None),
        ],
        Route::PostsViewScreen { id } => vec![
            ("posts".to_string(), Some(Route::PostsListScreen {})),
            (id.to_string(), None),
        ],
        Route::PostsListScreen {} => vec![("posts".to_string(), None)],
        Route::CategoriesAddScreen {} => vec![
            (
                "categories".to_string(),
                Some(Route::CategoriesListScreen {}),
            ),
            ("add".to_string(), None),
        ],
        Route::CategoriesListScreen {} => vec![("categories".to_string(), None)],
        Route::CategoriesEditScreen { id } => vec![
            (
                "categories".to_string(),
                Some(Route::CategoriesListScreen {}),
            ),
            (id.to_string(), None),
            ("edit".to_string(), None),
        ],
        Route::TagsAddScreen {} => vec![
            ("tags".to_string(), Some(Route::TagsListScreen {})),
            ("add".to_string(), None),
        ],
        Route::TagsEditScreen { id } => vec![
            ("tags".to_string(), Some(Route::TagsListScreen {})),
            (id.to_string(), None),
            ("edit".to_string(), None),
        ],
        Route::TagsListScreen {} => vec![("tags".to_string(), None)],
        Route::MediaUploadScreen {} => vec![
            ("media".to_string(), Some(Route::MediaListScreen {})),
            ("upload".to_string(), None),
        ],
        Route::MediaListScreen {} => vec![("media".to_string(), None)],
        Route::UsersAddScreen {} => vec![
            ("users".to_string(), Some(Route::UsersListScreen {})),
            ("add".to_string(), None),
        ],
        Route::UsersEditScreen { id } => vec![
            ("users".to_string(), Some(Route::UsersListScreen {})),
            (id.to_string(), None),
            ("edit".to_string(), None),
        ],
        Route::UsersListScreen {} => vec![("users".to_string(), None)],
        Route::SonnerDemoScreen {} => {
            vec![("demo".to_string(), None), ("sonner".to_string(), None)]
        }
        Route::AnalyticsScreen {} => vec![("analytics".to_string(), None)],
        Route::HomeScreen {} | Route::LoginScreen {} => vec![],
    };

    let container_class = props
        .class
        .clone()
        .unwrap_or_else(|| "container mx-auto px-4 py-6 md:py-8".to_string());

    let segments_elements: Vec<Element> = segments
        .iter()
        .enumerate()
        .map(|(i, (text, link_route))| {
            let separator = if i < segments.len() - 1 {
                rsx! { BreadcrumbSeparator {} }
            } else {
                rsx! {}
            };
            if let Some(route_ref) = link_route {
                let r = route_ref.clone();
                rsx! {
                    BreadcrumbItem {
                        BreadcrumbLink {
                            onclick: Some(Callback::new(move |_| { let _ = nav.push(r.clone()); })),
                            "{text}"
                        }
                    }
                    {separator}
                }
            } else {
                rsx! {
                    BreadcrumbItem { BreadcrumbPage { "{text}" } }
                    {separator}
                }
            }
        })
        .collect();

    if props.embedded {
        rsx! {
            // Breadcrumb
            Breadcrumb {
                BreadcrumbList {
                    // Dashboard root
                    BreadcrumbItem {
                        BreadcrumbLink {
                            onclick: Some(Callback::new(move |_| { nav.push(Route::HomeScreen {}); })),
                            "Dashboard"
                        }
                    }
                    BreadcrumbSeparator {}

                    // Segments
                    for element in &segments_elements { {element} }
                }
            }

            // Header row
            div { class: "mt-6 flex flex-col items-start justify-between gap-6 md:flex-row md:items-center",
                div { class: "space-y-2",
                    h1 { class: "text-3xl md:text-4xl font-bold tracking-tight", "{props.title}" }
                    p { class: "text-sm md:text-base text-zinc-600 dark:text-zinc-400", "{props.description}" }
                }
                div { class: "flex items-center gap-2",
                    if let Some(actions) = props.actions.clone() { {actions} }
                }
            }
        }
    } else {
        rsx! {
            // Top region with breadcrumb and header
            div { class: "border-b border-border/60 bg-transparent transition-colors duration-300",
                div { class: container_class,
                    // Breadcrumb
                    Breadcrumb {
                        BreadcrumbList {
                            // Dashboard root
                            BreadcrumbItem {
                                BreadcrumbLink {
                                    // href can be omitted; we handle nav in onclick
                                    onclick: Some(Callback::new(move |_| { nav.push(Route::HomeScreen {}); })),
                                    "Dashboard"
                                }
                            }
                            BreadcrumbSeparator {}

                            // Segments
                            for element in &segments_elements { {element} }
                        }
                    }

                    // Header row
                    div { class: "mt-6 flex flex-col items-start justify-between gap-6 md:flex-row md:items-center",
                        div { class: "space-y-2",
                            h1 { class: "text-3xl md:text-4xl font-bold tracking-tight", "{props.title}" }
                            p { class: "text-sm md:text-base text-zinc-600 dark:text-zinc-400", "{props.description}" }
                        }
                        div { class: "flex items-center gap-2",
                            if let Some(actions) = props.actions.clone() { {actions} }
                        }
                    }
                }
            }
        }
    }
}
