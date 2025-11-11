use crate::store::{use_image_editor, EditorTool};
use crate::ui::shadcn::{Button, ButtonSize, ButtonVariant};
use dioxus::prelude::*;
use hmziq_dioxus_free_icons::{
    icons::ld_icons::{LdCrop, LdMaximize, LdRotateCw, LdSettings},
    Icon,
};

/// Toolbar with editing tool buttons
#[component]
pub fn EditorToolbar() -> Element {
    let editor = use_image_editor();
    let active_tool = editor.active_tool.read().clone();

    rsx! {
        div {
            class: "flex items-center gap-2 p-2 rounded-lg border border-zinc-200 dark:border-zinc-800",

            // Crop button
            Button {
                variant: if active_tool == EditorTool::Crop { ButtonVariant::Default } else { ButtonVariant::Outline },
                size: ButtonSize::Sm,
                onclick: move |_| {
                    editor.select_tool(
                        if active_tool == EditorTool::Crop { EditorTool::None } else { EditorTool::Crop }
                    );
                },
                class: "flex items-center gap-2",
                Icon { icon: LdCrop {}, width: 16, height: 16 }
                "Crop"
            }

            // Resize button
            Button {
                variant: if active_tool == EditorTool::Resize { ButtonVariant::Default } else { ButtonVariant::Outline },
                size: ButtonSize::Sm,
                onclick: move |_| {
                    editor.select_tool(
                        if active_tool == EditorTool::Resize { EditorTool::None } else { EditorTool::Resize }
                    );
                },
                class: "flex items-center gap-2",
                Icon { icon: LdMaximize {}, width: 16, height: 16 }
                "Resize"
            }

            // Rotate button
            Button {
                variant: if active_tool == EditorTool::Rotate { ButtonVariant::Default } else { ButtonVariant::Outline },
                size: ButtonSize::Sm,
                onclick: move |_| {
                    editor.select_tool(
                        if active_tool == EditorTool::Rotate { EditorTool::None } else { EditorTool::Rotate }
                    );
                },
                class: "flex items-center gap-2",
                Icon { icon: LdRotateCw {}, width: 16, height: 16 }
                "Rotate"
            }

            // Compress button
            Button {
                variant: if active_tool == EditorTool::Compress { ButtonVariant::Default } else { ButtonVariant::Outline },
                size: ButtonSize::Sm,
                onclick: move |_| {
                    editor.select_tool(
                        if active_tool == EditorTool::Compress { EditorTool::None } else { EditorTool::Compress }
                    );
                },
                class: "flex items-center gap-2",
                Icon { icon: LdSettings {}, width: 16, height: 16 }
                "Compress"
            }
        }
    }
}
