use dioxus::prelude::*;
use ruxlog_shared::store::Post;

/// Auto-generated table of contents from EditorJS header blocks.
#[component]
pub fn TableOfContents(post: Post) -> Element {
    let mut headings: Vec<(String, String, u8)> = Vec::new(); // (id, text, level)

    for block in &post.content.blocks {
        if let ruxlog_shared::EditorJsBlock::Header { data, .. } = block {
            let level = data.level;
            if (2..=4).contains(&level) {
                let id = slugify(&data.text);
                headings.push((id, data.text.clone(), level));
            }
        }
    }

    if headings.is_empty() {
        return rsx! {};
    }

    rsx! {
        nav { class: "rounded-lg border border-border bg-card p-4 mb-8",
            h2 { class: "text-sm font-semibold mb-3 text-muted-foreground uppercase tracking-wide", "Table of Contents" }
            ul { class: "space-y-1.5",
                for (id, text, level) in headings.iter() {
                    {
                        let indent = match level {
                            2 => "",
                            3 => "ml-3",
                            4 => "ml-6",
                            _ => "",
                        };
                        rsx! {
                            li { class: "{indent}",
                                a {
                                    href: "#{id}",
                                    class: "text-sm text-muted-foreground hover:text-primary transition-colors block truncate",
                                    dangerous_inner_html: "{text}",
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn slugify(text: &str) -> String {
    let re = regex::Regex::new(r"<[^>]*>").unwrap();
    let clean = re.replace_all(text, "");
    let clean = clean.trim().to_lowercase();
    clean
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
