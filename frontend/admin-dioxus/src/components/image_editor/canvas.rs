use crate::store::{use_image_editor, EditorTool};
use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct EditorCanvasProps {
    pub blob_url: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, PartialEq)]
enum DragMode {
    None,
    Move,
    ResizeNW,
    ResizeNE,
    ResizeSW,
    ResizeSE,
}

/// Canvas component for displaying the image being edited
#[component]
pub fn EditorCanvas(props: EditorCanvasProps) -> Element {
    let editor = use_image_editor();
    let active_tool = *editor.active_tool.read();
    let crop_region = editor.crop_region.read().clone();

    // Drag state
    let mut drag_mode = use_signal(|| DragMode::None);
    let mut drag_start_x = use_signal(|| 0.0);
    let mut drag_start_y = use_signal(|| 0.0);
    let mut initial_crop = use_signal(|| crop_region.clone());

    // Calculate display dimensions (max 800px width while maintaining aspect ratio)
    let max_display_width = 800.0;
    let aspect_ratio = props.width as f64 / props.height as f64;
    let (display_width, display_height) = if props.width as f64 > max_display_width {
        let width = max_display_width;
        let height = width / aspect_ratio;
        (width, height)
    } else {
        (props.width as f64, props.height as f64)
    };

    // Scale factor for crop overlay
    let scale_x = display_width / props.width as f64;
    let scale_y = display_height / props.height as f64;

    // Mouse move handler
    let handle_mouse_move = move |evt: Event<MouseData>| {
        if *drag_mode.read() == DragMode::None {
            return;
        }

        let dx = evt.page_coordinates().x - *drag_start_x.read();
        let dy = evt.page_coordinates().y - *drag_start_y.read();

        let initial = initial_crop.read().clone();
        let mut new_crop = initial.clone();

        match *drag_mode.read() {
            DragMode::Move => {
                // Move the crop region
                let new_x = (initial.x as f64 + dx / scale_x)
                    .max(0.0)
                    .min((props.width - initial.width) as f64);
                let new_y = (initial.y as f64 + dy / scale_y)
                    .max(0.0)
                    .min((props.height - initial.height) as f64);
                new_crop.x = new_x as u32;
                new_crop.y = new_y as u32;
            }
            DragMode::ResizeNW => {
                // Resize from top-left corner
                let new_x = (initial.x as f64 + dx / scale_x)
                    .max(0.0)
                    .min((initial.x + initial.width - 10) as f64);
                let new_y = (initial.y as f64 + dy / scale_y)
                    .max(0.0)
                    .min((initial.y + initial.height - 10) as f64);
                new_crop.x = new_x as u32;
                new_crop.y = new_y as u32;
                new_crop.width =
                    ((initial.x + initial.width) as i32 - new_crop.x as i32).max(10) as u32;
                new_crop.height =
                    ((initial.y + initial.height) as i32 - new_crop.y as i32).max(10) as u32;
            }
            DragMode::ResizeNE => {
                // Resize from top-right corner
                let new_y = (initial.y as f64 + dy / scale_y)
                    .max(0.0)
                    .min((initial.y + initial.height - 10) as f64);
                let new_width = ((initial.width as f64 + dx / scale_x)
                    .max(10.0)
                    .min((props.width - initial.x) as f64)) as u32;
                new_crop.y = new_y as u32;
                new_crop.width = new_width;
                new_crop.height =
                    ((initial.y + initial.height) as i32 - new_crop.y as i32).max(10) as u32;
            }
            DragMode::ResizeSW => {
                // Resize from bottom-left corner
                let new_x = (initial.x as f64 + dx / scale_x)
                    .max(0.0)
                    .min((initial.x + initial.width - 10) as f64);
                let new_height = ((initial.height as f64 + dy / scale_y)
                    .max(10.0)
                    .min((props.height - initial.y) as f64))
                    as u32;
                new_crop.x = new_x as u32;
                new_crop.width =
                    ((initial.x + initial.width) as i32 - new_crop.x as i32).max(10) as u32;
                new_crop.height = new_height;
            }
            DragMode::ResizeSE => {
                // Resize from bottom-right corner
                let new_width = ((initial.width as f64 + dx / scale_x)
                    .max(10.0)
                    .min((props.width - initial.x) as f64)) as u32;
                let new_height = ((initial.height as f64 + dy / scale_y)
                    .max(10.0)
                    .min((props.height - initial.y) as f64))
                    as u32;
                new_crop.width = new_width;
                new_crop.height = new_height;
            }
            DragMode::None => {}
        }

        *editor.crop_region.write() = new_crop;
    };

    // Mouse up handler
    let handle_mouse_up = move |_evt: Event<MouseData>| {
        drag_mode.set(DragMode::None);
    };

    // Start dragging crop region
    let start_move = move |evt: Event<MouseData>| {
        evt.stop_propagation();
        drag_mode.set(DragMode::Move);
        drag_start_x.set(evt.page_coordinates().x);
        drag_start_y.set(evt.page_coordinates().y);
        initial_crop.set(editor.crop_region.read().clone());
    };

    // Start resizing from corners
    let start_resize_nw = move |evt: Event<MouseData>| {
        evt.stop_propagation();
        drag_mode.set(DragMode::ResizeNW);
        drag_start_x.set(evt.page_coordinates().x);
        drag_start_y.set(evt.page_coordinates().y);
        initial_crop.set(editor.crop_region.read().clone());
    };

    let start_resize_ne = move |evt: Event<MouseData>| {
        evt.stop_propagation();
        drag_mode.set(DragMode::ResizeNE);
        drag_start_x.set(evt.page_coordinates().x);
        drag_start_y.set(evt.page_coordinates().y);
        initial_crop.set(editor.crop_region.read().clone());
    };

    let start_resize_sw = move |evt: Event<MouseData>| {
        evt.stop_propagation();
        drag_mode.set(DragMode::ResizeSW);
        drag_start_x.set(evt.page_coordinates().x);
        drag_start_y.set(evt.page_coordinates().y);
        initial_crop.set(editor.crop_region.read().clone());
    };

    let start_resize_se = move |evt: Event<MouseData>| {
        evt.stop_propagation();
        drag_mode.set(DragMode::ResizeSE);
        drag_start_x.set(evt.page_coordinates().x);
        drag_start_y.set(evt.page_coordinates().y);
        initial_crop.set(editor.crop_region.read().clone());
    };

    rsx! {
        div {
            class: "relative flex items-center justify-center rounded-lg overflow-hidden border border-zinc-200 dark:border-zinc-800",
            style: "min-height: 400px;",
            onmousemove: handle_mouse_move,
            onmouseup: handle_mouse_up,
            onmouseleave: handle_mouse_up,

            // Image display
            img {
                src: "{props.blob_url}",
                alt: "Image being edited",
                class: "max-w-full h-auto",
                style: "width: {display_width}px; height: {display_height}px; object-fit: contain;",
            }

            // Crop overlay
            if active_tool == EditorTool::Crop {
                div {
                    class: "absolute border-2 border-blue-500 bg-blue-500/20 cursor-move",
                    style: "left: {crop_region.x as f64 * scale_x}px; top: {crop_region.y as f64 * scale_y}px; width: {crop_region.width as f64 * scale_x}px; height: {crop_region.height as f64 * scale_y}px;",
                    onmousedown: start_move,

                    // Resize handles
                    div {
                        class: "absolute w-3 h-3 bg-blue-500 rounded-full",
                        style: "left: -6px; top: -6px; cursor: nw-resize;",
                        onmousedown: start_resize_nw,
                    }
                    div {
                        class: "absolute w-3 h-3 bg-blue-500 rounded-full",
                        style: "right: -6px; top: -6px; cursor: ne-resize;",
                        onmousedown: start_resize_ne,
                    }
                    div {
                        class: "absolute w-3 h-3 bg-blue-500 rounded-full",
                        style: "left: -6px; bottom: -6px; cursor: sw-resize;",
                        onmousedown: start_resize_sw,
                    }
                    div {
                        class: "absolute w-3 h-3 bg-blue-500 rounded-full",
                        style: "right: -6px; bottom: -6px; cursor: se-resize;",
                        onmousedown: start_resize_se,
                    }
                }
            }
        }
    }
}
