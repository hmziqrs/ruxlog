use dioxus::prelude::*;

use crate::containers::page_header::PageHeader;
use oxui::shadcn::badge::{Badge, BadgeVariant};
use oxui::shadcn::button::{Button, ButtonVariant};

// TODO: Replace mock data with real API call to GET /admin/v1/audit-logs
// The backend has an audit_logs table (see backend/api/src/db/sea_models/audit_log/model.rs)
// but no admin endpoint exists yet. Once the endpoint is created, swap in a store call
// similar to how billing screens use `use_billing()`.

#[derive(Debug, Clone, PartialEq)]
pub struct AuditLogEntry {
    pub id: i64,
    pub user_id: Option<i32>,
    pub user_name: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: String,
    pub ip_address: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditLogPage {
    pub items: Vec<AuditLogEntry>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

const PER_PAGE: i64 = 20;

/// All known action types for the filter dropdown.
const ACTION_OPTIONS: &[&str] = &[
    "All",
    "user.login",
    "user.logout",
    "user.create",
    "user.update",
    "user.delete",
    "post.create",
    "post.update",
    "post.delete",
    "plan.create",
    "plan.update",
    "plan.delete",
];

fn mock_data() -> Vec<AuditLogEntry> {
    vec![
        AuditLogEntry {
            id: 1,
            user_id: Some(1),
            user_name: Some("admin".to_string()),
            action: "user.login".to_string(),
            resource_type: "user".to_string(),
            resource_id: "1".to_string(),
            ip_address: Some("192.168.1.100".to_string()),
            created_at: "2026-05-12T09:15:30.000".to_string(),
        },
        AuditLogEntry {
            id: 2,
            user_id: Some(1),
            user_name: Some("admin".to_string()),
            action: "post.create".to_string(),
            resource_type: "post".to_string(),
            resource_id: "42".to_string(),
            ip_address: Some("192.168.1.100".to_string()),
            created_at: "2026-05-12T10:22:45.000".to_string(),
        },
        AuditLogEntry {
            id: 3,
            user_id: Some(2),
            user_name: Some("editor".to_string()),
            action: "post.update".to_string(),
            resource_type: "post".to_string(),
            resource_id: "42".to_string(),
            ip_address: Some("10.0.0.55".to_string()),
            created_at: "2026-05-12T11:05:12.000".to_string(),
        },
        AuditLogEntry {
            id: 4,
            user_id: Some(1),
            user_name: Some("admin".to_string()),
            action: "user.create".to_string(),
            resource_type: "user".to_string(),
            resource_id: "3".to_string(),
            ip_address: Some("192.168.1.100".to_string()),
            created_at: "2026-05-11T14:30:00.000".to_string(),
        },
        AuditLogEntry {
            id: 5,
            user_id: None,
            user_name: None,
            action: "user.login".to_string(),
            resource_type: "user".to_string(),
            resource_id: "0".to_string(),
            ip_address: Some("203.0.113.7".to_string()),
            created_at: "2026-05-11T08:12:05.000".to_string(),
        },
    ]
}

#[component]
pub fn AuditLogViewerScreen() -> Element {
    let mut current_page = use_signal(|| 1i64);
    let mut action_filter = use_signal(|| "All".to_string());
    let mut user_search = use_signal(|| String::new());
    let mut date_from = use_signal(|| String::new());
    let mut date_to = use_signal(|| String::new());

    // TODO: Replace with real API data fetching
    let all_entries = use_signal(|| mock_data());

    // Apply filters
    let filtered = use_memo(move || {
        let entries = all_entries.read();
        let action = action_filter.read();
        let search = user_search.read();
        let from = date_from.read();
        let to = date_to.read();

        entries
            .iter()
            .filter(|e| {
                // Action filter
                if action.as_str() != "All" && e.action != *action {
                    return false;
                }
                // User search
                if !search.is_empty() {
                    let s = search.to_lowercase();
                    let matches = e
                        .user_name
                        .as_deref()
                        .map(|n| n.to_lowercase().contains(&s))
                        .unwrap_or(false)
                        || e
                            .user_id
                            .map(|id| id.to_string().contains(&s))
                            .unwrap_or(false);
                    if !matches {
                        return false;
                    }
                }
                // Date from
                if !from.is_empty() && !e.created_at.starts_with(from.as_str()) {
                    return false;
                }
                // Date to
                if !to.is_empty() {
                    if let Some(entry_date) = e.created_at.get(..10) {
                        if entry_date > to.as_str() {
                            return false;
                        }
                    }
                }
                true
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    let total = filtered.read().len() as i64;
    let total_pages = if total == 0 {
        1
    } else {
        (total + PER_PAGE - 1) / PER_PAGE
    };

    // Clamp current page
    {
        let page = current_page();
        if page < 1 {
            current_page.set(1);
        } else if page > total_pages {
            current_page.set(total_pages);
        }
    }

    let page_items = use_memo(move || {
        let items = filtered.read();
        let p = current_page();
        let start = ((p - 1) * PER_PAGE) as usize;
        let end = (start + PER_PAGE as usize).min(items.len());
        if start >= items.len() {
            vec![]
        } else {
            items[start..end].to_vec()
        }
    });

    let go_prev = move |_| {
        let p = current_page();
        if p > 1 {
            current_page.set(p - 1);
        }
    };

    let go_next = move |_| {
        let p = current_page();
        if p < total_pages {
            current_page.set(p + 1);
        }
    };

    rsx! {
        PageHeader {
            title: "Audit Logs".to_string(),
            description: "Track administrative actions and system events.".to_string(),
        }

        div { class: "container mx-auto px-4 py-6 md:py-8 space-y-6",
            // Filter bar
            div { class: "flex flex-wrap items-end gap-3",
                // Action type dropdown
                div { class: "flex flex-col gap-1",
                    label { class: "text-xs font-medium text-muted-foreground", "Action" }
                    select {
                        class: "h-9 rounded-md border border-zinc-200 dark:border-zinc-800 bg-background px-3 text-sm focus:outline-none focus:ring-2 focus:ring-ring",
                        value: "{action_filter}",
                        onchange: move |e| {
                            action_filter.set(e.value());
                            current_page.set(1);
                        },
                        for option in ACTION_OPTIONS.iter() {
                            option { value: *option, selected: action_filter.read().as_str() == *option, "{option}" }
                        }
                    }
                }

                // User search
                div { class: "flex flex-col gap-1",
                    label { class: "text-xs font-medium text-muted-foreground", "User" }
                    input {
                        r#type: "text",
                        class: "h-9 w-48 rounded-md border border-zinc-200 dark:border-zinc-800 bg-background px-3 text-sm focus:outline-none focus:ring-2 focus:ring-ring",
                        placeholder: "Search user...",
                        value: "{user_search}",
                        oninput: move |e| {
                            user_search.set(e.value());
                            current_page.set(1);
                        },
                    }
                }

                // Date from
                div { class: "flex flex-col gap-1",
                    label { class: "text-xs font-medium text-muted-foreground", "From" }
                    input {
                        r#type: "date",
                        class: "h-9 rounded-md border border-zinc-200 dark:border-zinc-800 bg-background px-3 text-sm focus:outline-none focus:ring-2 focus:ring-ring",
                        value: "{date_from}",
                        oninput: move |e| {
                            date_from.set(e.value());
                            current_page.set(1);
                        },
                    }
                }

                // Date to
                div { class: "flex flex-col gap-1",
                    label { class: "text-xs font-medium text-muted-foreground", "To" }
                    input {
                        r#type: "date",
                        class: "h-9 rounded-md border border-zinc-200 dark:border-zinc-800 bg-background px-3 text-sm focus:outline-none focus:ring-2 focus:ring-ring",
                        value: "{date_to}",
                        oninput: move |e| {
                            date_to.set(e.value());
                            current_page.set(1);
                        },
                    }
                }

                // Clear filters
                Button {
                    variant: ButtonVariant::Outline,
                    class: "h-9",
                    onclick: move |_| {
                        action_filter.set("All".to_string());
                        user_search.set(String::new());
                        date_from.set(String::new());
                        date_to.set(String::new());
                        current_page.set(1);
                    },
                    "Clear"
                }
            }

            // Results summary
            div { class: "text-sm text-muted-foreground",
                {
                    let suffix = if total != 1 { "s" } else { "" };
                    format!("{} event{} found", total, suffix)
                }
            }

            // Table
            div { class: "rounded-lg border border-zinc-200 dark:border-zinc-800 overflow-hidden",
                table { class: "w-full",
                    thead { class: "bg-muted/50",
                        tr {
                            th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Timestamp" }
                            th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "User" }
                            th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Action" }
                            th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Resource Type" }
                            th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "Resource ID" }
                            th { class: "py-2 px-3 text-left font-medium text-xs md:text-sm", "IP Address" }
                        }
                    }
                    tbody {
                        for entry in page_items.read().iter() {
                            { audit_log_row(entry) }
                        }
                    }
                }
            }

            // Pagination
            div { class: "flex items-center justify-between",
                p { class: "text-sm text-muted-foreground",
                    "Page {current_page} of {total_pages}"
                }
                div { class: "flex items-center gap-2",
                    Button {
                        variant: ButtonVariant::Outline,
                        class: "h-8",
                        disabled: *current_page.read() <= 1,
                        onclick: go_prev,
                        "Previous"
                    }
                    Button {
                        variant: ButtonVariant::Outline,
                        class: "h-8",
                        disabled: *current_page.read() >= total_pages,
                        onclick: go_next,
                        "Next"
                    }
                }
            }
        }
    }
}

fn audit_log_row(entry: &AuditLogEntry) -> Element {
    let action = entry.action.clone();

    let action_badge = match action.as_str() {
        a if a.starts_with("user.") => rsx! {
            Badge { class: "bg-blue-100 text-blue-800 border-blue-200 dark:bg-blue-900/20 dark:text-blue-400",
                "{action}"
            }
        },
        a if a.starts_with("post.") => rsx! {
            Badge { class: "bg-green-100 text-green-800 border-green-200 dark:bg-green-900/20 dark:text-green-400",
                "{action}"
            }
        },
        a if a.starts_with("plan.") => rsx! {
            Badge { class: "bg-purple-100 text-purple-800 border-purple-200 dark:bg-purple-900/20 dark:text-purple-400",
                "{action}"
            }
        },
        _ => rsx! {
            Badge { variant: BadgeVariant::Secondary, "{action}" }
        },
    };

    rsx! {
        tr { class: "border-b border-zinc-200 dark:border-zinc-800 hover:bg-muted/30 transition-colors",
            td { class: "py-2 px-3 text-xs md:text-sm text-muted-foreground whitespace-nowrap",
                "{crate::utils::dates::format_short_date(&entry.created_at)}"
            }
            td { class: "py-2 px-3 text-xs md:text-sm",
                {entry.user_name.clone().unwrap_or_else(|| format!("#{}", entry.user_id.unwrap_or(0)))}
            }
            td { class: "py-2 px-3 text-xs md:text-sm", {action_badge} }
            td { class: "py-2 px-3 text-xs md:text-sm", "{entry.resource_type}" }
            td { class: "py-2 px-3 text-xs md:text-sm text-muted-foreground", "{entry.resource_id}" }
            td { class: "py-2 px-3 text-xs md:text-sm text-muted-foreground font-mono",
                {entry.ip_address.clone().unwrap_or_else(|| "--".to_string())}
            }
        }
    }
}
