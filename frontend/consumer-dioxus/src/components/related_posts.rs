use crate::components::post_card::{estimate_reading_time, PostCard};
use crate::server_fns::fetch_posts_by_category;
use dioxus::prelude::*;
use ruxlog_shared::store::Post;

#[component]
pub fn RelatedPosts(category_id: i32, current_post_id: i32) -> Element {
    let mut posts = use_signal(Vec::<Post>::new);
    let mut loaded = use_signal(|| false);

    use_effect(move || {
        if loaded() {
            return;
        }
        let cat_id = category_id;
        spawn(async move {
            match fetch_posts_by_category(cat_id).await {
                Ok(result) => {
                    let filtered: Vec<Post> = result
                        .data
                        .into_iter()
                        .filter(|p| p.id != current_post_id)
                        .take(3)
                        .collect();
                    posts.set(filtered);
                    loaded.set(true);
                }
                Err(_) => {
                    loaded.set(true);
                }
            }
        });
    });

    if posts().is_empty() {
        return rsx! {};
    }

    rsx! {
        section { class: "mt-12 pt-8 border-t border-border",
            h2 { class: "text-xl font-bold mb-6", "Related Posts" }
            div { class: "grid md:grid-cols-3 gap-6",
                for post in posts().iter() {
                    PostCard { key: "{post.id}", post: post.clone(), is_premium: false }
                }
            }
        }
    }
}
