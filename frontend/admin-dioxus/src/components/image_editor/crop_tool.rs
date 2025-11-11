use crate::store::use_image_editor;
use crate::ui::shadcn::Button;
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum AspectRatio {
    Free,
    Square,    // 1:1
    Landscape, // 16:9
    Portrait,  // 9:16
    FourThree, // 4:3
    ThreeFour, // 3:4
    ThreeTwo,  // 3:2
    TwoThree,  // 2:3
}

impl AspectRatio {
    fn ratio(&self) -> Option<f64> {
        match self {
            AspectRatio::Free => None,
            AspectRatio::Square => Some(1.0),
            AspectRatio::Landscape => Some(16.0 / 9.0),
            AspectRatio::Portrait => Some(9.0 / 16.0),
            AspectRatio::FourThree => Some(4.0 / 3.0),
            AspectRatio::ThreeFour => Some(3.0 / 4.0),
            AspectRatio::ThreeTwo => Some(3.0 / 2.0),
            AspectRatio::TwoThree => Some(2.0 / 3.0),
        }
    }

    fn label(&self) -> &'static str {
        match self {
            AspectRatio::Free => "Free",
            AspectRatio::Square => "1:1",
            AspectRatio::Landscape => "16:9",
            AspectRatio::Portrait => "9:16",
            AspectRatio::FourThree => "4:3",
            AspectRatio::ThreeFour => "3:4",
            AspectRatio::ThreeTwo => "3:2",
            AspectRatio::TwoThree => "2:3",
        }
    }
}

/// Crop tool controls
#[component]
pub fn CropTool() -> Element {
    let editor = use_image_editor();
    let is_processing = *editor.is_processing.read();
    let mut selected_ratio = use_signal(|| AspectRatio::Free);

    let handle_apply = move |_| {
        spawn(async move {
            let _ = editor.apply_crop().await;
        });
    };

    let mut apply_aspect_ratio = move |ratio: AspectRatio| {
        selected_ratio.set(ratio);

        if let Some(ratio_val) = ratio.ratio() {
            let mut crop = editor.crop_region.write();
            let session = editor.current_session.read();

            if let Some(ref sess) = *session {
                // Calculate new dimensions maintaining the aspect ratio
                // Keep the current width and adjust height, or vice versa
                let current_ratio = crop.width as f64 / crop.height as f64;

                if current_ratio > ratio_val {
                    // Too wide, adjust width
                    crop.width = (crop.height as f64 * ratio_val) as u32;
                } else {
                    // Too tall, adjust height
                    crop.height = (crop.width as f64 / ratio_val) as u32;
                }

                // Ensure crop stays within bounds
                if crop.x + crop.width > sess.width {
                    crop.width = sess.width - crop.x;
                    crop.height = (crop.width as f64 / ratio_val) as u32;
                }
                if crop.y + crop.height > sess.height {
                    crop.height = sess.height - crop.y;
                    crop.width = (crop.height as f64 * ratio_val) as u32;
                }
            }
        }
    };

    const RATIOS: [AspectRatio; 8] = [
        AspectRatio::Free,
        AspectRatio::Square,
        AspectRatio::Landscape,
        AspectRatio::Portrait,
        AspectRatio::FourThree,
        AspectRatio::ThreeFour,
        AspectRatio::ThreeTwo,
        AspectRatio::TwoThree,
    ];

    rsx! {
        div {
            class: "space-y-4 p-4 rounded-lg border border-zinc-200 dark:border-zinc-800",

            h3 { class: "text-sm font-semibold mb-2", "Crop Settings" }

            // Aspect ratio buttons
            div { class: "space-y-2",
                label { class: "block text-xs text-muted-foreground", "Aspect Ratio" }
                div { class: "grid grid-cols-4 gap-2",
                    {
                        RATIOS.iter().map(|ratio| {
                            let is_selected = *selected_ratio.read() == *ratio;
                            let ratio_copy = *ratio;

                            rsx! {
                                button {
                                    key: "{ratio.label()}",
                                    r#type: "button",
                                    class: format!(
                                        "px-2 py-1 text-xs rounded border transition-colors {}",
                                        if is_selected {
                                            "border-blue-500 bg-blue-500 text-white"
                                        } else {
                                            "border-zinc-200 dark:border-zinc-800 hover:border-blue-500 hover:bg-blue-50 dark:hover:bg-blue-950"
                                        }
                                    ),
                                    onclick: move |_| apply_aspect_ratio(ratio_copy),
                                    "{ratio.label()}"
                                }
                            }
                        })
                    }
                }
            }

            div { class: "grid grid-cols-2 gap-3",
                // X position
                div {
                    label { class: "block text-xs text-muted-foreground mb-1", "X Position" }
                    input {
                        r#type: "number",
                        class: "w-full px-2 py-1 text-sm border border-zinc-200 dark:border-zinc-800 rounded",
                        value: "{editor.crop_region.read().x}",
                        min: 0,
                        oninput: move |e| {
                            if let Ok(val) = e.value().parse::<u32>() {
                                editor.crop_region.write().x = val;
                            }
                        }
                    }
                }

                // Y position
                div {
                    label { class: "block text-xs text-muted-foreground mb-1", "Y Position" }
                    input {
                        r#type: "number",
                        class: "w-full px-2 py-1 text-sm border border-zinc-200 dark:border-zinc-800 rounded",
                        value: "{editor.crop_region.read().y}",
                        min: 0,
                        oninput: move |e| {
                            if let Ok(val) = e.value().parse::<u32>() {
                                editor.crop_region.write().y = val;
                            }
                        }
                    }
                }

                // Width
                div {
                    label { class: "block text-xs text-muted-foreground mb-1", "Width" }
                    input {
                        r#type: "number",
                        class: "w-full px-2 py-1 text-sm border border-zinc-200 dark:border-zinc-800 rounded",
                        value: "{editor.crop_region.read().width}",
                        min: 1,
                        oninput: move |e| {
                            if let Ok(val) = e.value().parse::<u32>() {
                                editor.crop_region.write().width = val;
                            }
                        }
                    }
                }

                // Height
                div {
                    label { class: "block text-xs text-muted-foreground mb-1", "Height" }
                    input {
                        r#type: "number",
                        class: "w-full px-2 py-1 text-sm border border-zinc-200 dark:border-zinc-800 rounded",
                        value: "{editor.crop_region.read().height}",
                        min: 1,
                        oninput: move |e| {
                            if let Ok(val) = e.value().parse::<u32>() {
                                editor.crop_region.write().height = val;
                            }
                        }
                    }
                }
            }

            Button {
                onclick: handle_apply,
                disabled: is_processing,
                class: "w-full",
                if is_processing { "Applying..." } else { "Apply Crop" }
            }
        }
    }
}
