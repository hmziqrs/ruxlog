use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use web_sys::File;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditorTool {
    None,
    Crop,
    Resize,
    Rotate,
    Compress,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CropRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Default for CropRegion {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 100,
            height: 100,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResizeParams {
    pub width: u32,
    pub height: u32,
    pub maintain_aspect_ratio: bool,
}

impl Default for ResizeParams {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            maintain_aspect_ratio: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RotateParams {
    pub angle: i32, // degrees: 90, 180, 270, or custom
}

impl Default for RotateParams {
    fn default() -> Self {
        Self { angle: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CompressParams {
    pub quality: u8, // 0-100
}

impl Default for CompressParams {
    fn default() -> Self {
        Self { quality: 85 }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EditSession {
    pub original_blob_url: String,
    pub current_blob_url: String,
    pub original_file: Option<File>,
    pub width: u32,
    pub height: u32,
    pub original_size: Option<usize>,
    pub current_size: Option<usize>,
}

pub struct ImageEditorState {
    // Editor visibility
    pub is_open: GlobalSignal<bool>,

    // Current editing session
    pub current_session: GlobalSignal<Option<EditSession>>,

    // Selected tool
    pub active_tool: GlobalSignal<EditorTool>,

    // Tool parameters
    pub crop_region: GlobalSignal<CropRegion>,
    pub resize_params: GlobalSignal<ResizeParams>,
    pub rotate_params: GlobalSignal<RotateParams>,
    pub compress_params: GlobalSignal<CompressParams>,

    // Processing state
    pub is_processing: GlobalSignal<bool>,
    pub error_message: GlobalSignal<Option<String>>,

    // Compression stats
    pub compression_savings: GlobalSignal<Option<(usize, usize)>>, // (original, current) in bytes
}

impl ImageEditorState {
    pub fn new() -> Self {
        Self {
            is_open: GlobalSignal::new(|| false),
            current_session: GlobalSignal::new(|| None),
            active_tool: GlobalSignal::new(|| EditorTool::None),
            crop_region: GlobalSignal::new(|| CropRegion::default()),
            resize_params: GlobalSignal::new(|| ResizeParams::default()),
            rotate_params: GlobalSignal::new(|| RotateParams::default()),
            compress_params: GlobalSignal::new(|| CompressParams::default()),
            is_processing: GlobalSignal::new(|| false),
            error_message: GlobalSignal::new(|| None),
            compression_savings: GlobalSignal::new(|| None),
        }
    }
}

static IMAGE_EDITOR_STATE: std::sync::OnceLock<ImageEditorState> = std::sync::OnceLock::new();

pub fn use_image_editor() -> &'static ImageEditorState {
    IMAGE_EDITOR_STATE.get_or_init(|| ImageEditorState::new())
}
