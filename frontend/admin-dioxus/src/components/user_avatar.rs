use crate::store::Media;
use dioxus::prelude::*;

#[component]
pub fn UserAvatar(
    name: String,
    avatar: Option<Media>,
    #[props(default = "h-8 w-8".to_string())] size: String,
    #[props(default = "text-xs".to_string())] text_size: String,
) -> Element {
    let initials = generate_avatar_fallback(&name);

    rsx! {
        if let Some(avatar_media) = avatar {
            img {
                src: "{avatar_media.file_url}",
                alt: "{name}",
                class: "{size} rounded-full object-cover ring-2 ring-black/5 dark:ring-white/10"
            }
        } else {
            div {
                class: "{size} rounded-full bg-transparent border-1 border-zinc-300 dark:border-zinc-700 flex items-center justify-center text-zinc-700 dark:text-zinc-300 {text_size} font-semibold",
                "{initials}"
            }
        }
    }
}

fn generate_avatar_fallback(name: &str) -> String {
    let initials: String = name
        .split_whitespace()
        .take(2)
        .filter_map(|word| word.chars().next())
        .collect::<String>()
        .to_uppercase();
    initials
}
