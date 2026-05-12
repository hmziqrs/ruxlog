use dioxus::prelude::*;

/// Displays a card indicating the post is part of a series, with a link to view the full series.
///
/// This component renders nothing when the post is not associated with a series.
/// It is designed to be expanded later when the backend includes series metadata
/// in the post response.
#[component]
pub fn SeriesNavigation(
    series_id: Option<i32>,
    series_title: Option<String>,
) -> Element {
    match (series_id, series_title) {
        (Some(_id), Some(title)) => {
            rsx! {
                div {
                    class: "rounded-xl border border-border bg-muted/30 p-6",
                    div { class: "flex flex-col gap-2",
                        p { class: "text-sm font-medium text-muted-foreground", "Part of a series" }
                        h3 { class: "text-lg font-semibold", "{title}" }
                        p {
                            class: "text-sm text-muted-foreground",
                            "This post is part of a series. Check back later for the full series listing."
                        }
                    }
                }
            }
        }
        _ => rsx! {},
    }
}
