mod circle;
mod provider;
mod utils;

pub use circle::*;
pub use provider::*;
pub use utils::*;

use dioxus::prelude::*;

#[component]
pub fn AnimatedGridBackground() -> Element {
    let ctx = use_grid_context();
    let mount_ctx = ctx.clone();
    let resize_ctx = ctx.clone();

    info!("render");

    rsx! {
        div {
            class: "pointer-events-none absolute inset-0 -z-10 bg-transparent",
            aria_hidden: "true",
            onmount: move |event| {
                mount_ctx.handle_mount(event.data());
            },
            onresize: move |_| {
                resize_ctx.handle_resize();
            },

            // Vertical lines
            {ctx.grid_data.read().vertical_lines.iter().map(|pos| {
                let offset = format!("{pos:.2}px");
                rsx! {
                    div {
                        key: "v-{pos}",
                        class: "absolute inset-y-0 border-l border-border transition-[left] duration-200",
                        style: format!("left: {offset}; opacity: 1.0;"),
                    }
                }
            })},

            // Horizontal lines
            {ctx.grid_data.read().horizontal_lines.iter().map(|pos| {
                let offset = format!("{pos:.2}px");
                rsx! {
                    div {
                        key: "h-{pos}",
                        class: "absolute inset-x-0 border-t border-border transition-[top] duration-200",
                        style: format!("top: {offset}; opacity: 1.0;"),
                    }
                }
            })},
        }
    }
}
