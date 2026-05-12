use dioxus::prelude::*;

#[component]
pub fn ImportExportScreen() -> Element {
    let mut import_type = use_signal(|| "posts".to_string());
    let mut exporting = use_signal(|| false);
    let mut message = use_signal(|| Option::<String>::None);

    let handle_export = move |_| {
        let kind = import_type();
        exporting.set(true);
        message.set(None);
        spawn(async move {
            let endpoint = match kind.as_str() {
                "users" => "/user/v1/export",
                "payments" => "/billing/v1/payments/export",
                _ => "/post/v1/export",
            };
            match oxcore::http::get(endpoint).send().await {
                Ok(resp) if (200..300).contains(&resp.status()) => {
                    message.set(Some(format!("{} exported successfully.", kind)));
                }
                _ => {
                    message.set(Some(format!("Failed to export {}. Not yet implemented on backend.", kind)));
                }
            }
            exporting.set(false);
        });
    };

    rsx! {
        div { class: "space-y-6",
            div {
                h2 { class: "text-2xl font-bold", "Import & Export" }
                p { class: "text-muted-foreground mt-1",
                    "Bulk import and export data in CSV format."
                }
            }

            if let Some(msg) = message() {
                div { class: "rounded-lg border border-border bg-muted/50 p-4 text-sm",
                    "{msg}"
                }
            }

            // Export section
            div { class: "rounded-xl border border-border p-6",
                h3 { class: "text-lg font-semibold mb-4", "Export Data" }
                p { class: "text-sm text-muted-foreground mb-4",
                    "Download data as CSV files for backup or migration."
                }
                div { class: "flex items-end gap-4",
                    div { class: "flex-1",
                        label { class: "text-sm font-medium mb-1 block", "Data Type" }
                        select {
                            class: "w-full rounded-lg border border-border bg-background px-3 py-2 text-sm",
                            value: "{import_type}",
                            onchange: move |e| import_type.set(e.value()),
                            option { value: "posts", "Posts" }
                            option { value: "categories", "Categories" }
                            option { value: "tags", "Tags" }
                            option { value: "users", "Users" }
                            option { value: "payments", "Payments" }
                        }
                    }
                    button {
                        class: "px-4 py-2 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:opacity-90 transition-opacity disabled:opacity-50",
                        disabled: exporting(),
                        onclick: handle_export,
                        if exporting() { "Exporting..." } else { "Export CSV" }
                    }
                }
            }

            // Import section
            div { class: "rounded-xl border border-border p-6",
                h3 { class: "text-lg font-semibold mb-4", "Import Data" }
                p { class: "text-sm text-muted-foreground mb-4",
                    "Upload CSV files to bulk import posts, categories, or tags."
                }
                div { class: "rounded-lg border-2 border-dashed border-border p-8 text-center",
                    p { class: "text-muted-foreground mb-2",
                        "Drag and drop CSV files here, or click to browse."
                    }
                    p { class: "text-xs text-muted-foreground",
                        "Supported: .csv files with proper column headers."
                    }
                }
            }

            // Format reference
            div { class: "rounded-xl border border-border p-6",
                h3 { class: "text-lg font-semibold mb-4", "CSV Format Reference" }
                div { class: "space-y-4 text-sm",
                    div {
                        h4 { class: "font-medium mb-1", "Posts CSV" }
                        code { class: "text-xs bg-muted px-2 py-1 rounded block",
                            "title,slug,content,excerpt,category,tags,status"
                        }
                    }
                    div {
                        h4 { class: "font-medium mb-1", "Categories CSV" }
                        code { class: "text-xs bg-muted px-2 py-1 rounded block",
                            "name,slug,description,color"
                        }
                    }
                    div {
                        h4 { class: "font-medium mb-1", "Tags CSV" }
                        code { class: "text-xs bg-muted px-2 py-1 rounded block",
                            "name,slug,description,color"
                        }
                    }
                }
            }
        }
    }
}
