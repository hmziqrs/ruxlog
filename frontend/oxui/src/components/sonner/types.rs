//! Sonner (Dioxus) Types — Phase 0/1 scaffold
//! These types mirror Sonner's React/TS API in a Dioxus-friendly way.
//! Implementation is minimal to keep compilation green; behaviors are added in later phases.

use dioxus::prelude::*;
use std::collections::BTreeMap;

/// Toast types supported by Sonner
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToastType {
    Success,
    Info,
    Warning,
    Error,
    Loading,
}

impl ToastType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ToastType::Success => "success",
            ToastType::Info => "info",
            ToastType::Warning => "warning",
            ToastType::Error => "error",
            ToastType::Loading => "loading",
        }
    }
}

/// Where the toasts should appear
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Position {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    TopCenter,
    BottomCenter,
}

/// Allowed swipe/drag directions (used for swipe-to-dismiss)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwipeDirection {
    Top,
    Right,
    Bottom,
    Left,
}

/// Optional per-side offset, or a single value (px/em/etc)
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Offset {
    /// Single numeric offset in pixels
    Number(i32),
    /// Arbitrary string like "24px", "2rem"
    Text(String),
    /// Individual sides override (strings to allow px/rem/% etc.)
    Sides {
        top: Option<String>,
        right: Option<String>,
        bottom: Option<String>,
        left: Option<String>,
    },
}

/// Action button attached to a toast
#[derive(Clone)]
pub struct Action {
    pub label: String,
    pub on_click: Option<Callback<MouseEvent>>,
    pub dismiss_toast: bool,
    pub action_button_style: Option<BTreeMap<String, String>>,
}

impl Action {
    pub fn new(
        label: String,
        on_click: Option<Callback<MouseEvent>>,
        action_button_style: Option<BTreeMap<String, String>>,
    ) -> Self {
        Self {
            label,
            on_click,
            dismiss_toast: false,
            action_button_style,
        }
    }

    pub fn with_on_click(label: String, on_click: Callback<MouseEvent>) -> Self {
        Self {
            label,
            on_click: Some(on_click),
            dismiss_toast: false,
            action_button_style: None,
        }
    }

    /// Set whether clicking the action should dismiss the toast
    pub fn with_dismiss_toast(mut self, dismiss: bool) -> Self {
        self.dismiss_toast = dismiss;
        self
    }
}

impl std::fmt::Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Action")
            .field("label", &self.label)
            .field("has_on_click", &self.on_click.is_some())
            .field("dismiss_toast", &self.dismiss_toast)
            .field("action_button_style", &self.action_button_style)
            .finish()
    }
}

/// Optional class names for fine-grained styling overrides
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ToastClassNames {
    pub toast: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub content: Option<String>,
    pub title: Option<String>,
    pub cancel_button: Option<String>,
    pub action_button: Option<String>,
    pub success: Option<String>,
    pub error: Option<String>,
    pub info: Option<String>,
    pub warning: Option<String>,
    pub loading: Option<String>,
    pub default: Option<String>,
}

/// Custom icons, represented as symbolic identifiers for Phase 1
/// Supported identifiers include: "none", "success", "info", "warning", "error",
/// "loading"/"loader", and "close". Unknown identifiers fall back to defaults.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ToastIcons {
    pub success: Option<String>,
    pub info: Option<String>,
    pub warning: Option<String>,
    pub error: Option<String>,
    pub loading: Option<String>,
    pub close: Option<String>,
}

/// Per-toast options (external API) similar to Sonner's toast options
#[derive(Clone, Debug, Default)]
pub struct ToastOptions {
    pub class_name: Option<String>,
    pub close_button: Option<bool>,
    pub description_class_name: Option<String>,
    pub cancel_button_style: Option<BTreeMap<String, String>>, // CSS-like style map
    pub action_button_style: Option<BTreeMap<String, String>>,
    pub duration_ms: Option<u64>,
    pub unstyled: Option<bool>,
    pub class_names: Option<ToastClassNames>,
    pub close_button_aria_label: Option<String>,
    pub toaster_id: Option<String>,
    pub on_auto_close: Option<Callback<u64>>, // Phase 3: public API
    pub on_dismiss: Option<Callback<u64>>, // Phase 9+: fired whenever the toast is removed (auto or manual)
    /// Per-toast icon override keyword (see ToastIcons docs)
    pub icon: Option<String>,
    /// Optional primary action button for the toast
    pub action: Option<Action>,
    /// Optional cancel/secondary action button for the toast
    pub cancel: Option<Action>,
}

impl ToastOptions {
    /// Set the duration in milliseconds (None = let provider default apply)
    pub fn with_duration(mut self, duration_ms: Option<u64>) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    /// Convenience: set a concrete duration
    pub fn duration(self, ms: u64) -> Self {
        self.with_duration(Some(ms))
    }

    /// Set or clear the close button flag
    pub fn with_close_button(mut self, close_button: Option<bool>) -> Self {
        self.close_button = close_button;
        self
    }

    /// Set the main class name
    pub fn with_class_name<S: Into<String>>(mut self, class_name: Option<S>) -> Self {
        self.class_name = class_name.map(Into::into);
        self
    }

    /// Set the description class name
    pub fn with_description_class_name<S: Into<String>>(mut self, class_name: Option<S>) -> Self {
        self.description_class_name = class_name.map(Into::into);
        self
    }

    /// Set per-part class names struct
    pub fn with_class_names(mut self, class_names: Option<ToastClassNames>) -> Self {
        self.class_names = class_names;
        self
    }

    /// Set the toaster id this toast should target
    pub fn with_toaster_id<S: Into<String>>(mut self, toaster_id: Option<S>) -> Self {
        self.toaster_id = toaster_id.map(Into::into);
        self
    }

    /// Set unstyled flag
    pub fn with_unstyled(mut self, unstyled: Option<bool>) -> Self {
        self.unstyled = unstyled;
        self
    }

    /// Set per-toast icon override keyword
    pub fn with_icon<S: Into<String>>(mut self, icon: Option<S>) -> Self {
        self.icon = icon.map(Into::into);
        self
    }

    /// Set the primary action button
    pub fn with_action(mut self, action: Option<Action>) -> Self {
        self.action = action;
        self
    }

    /// Set the cancel/secondary action button
    pub fn with_cancel(mut self, cancel: Option<Action>) -> Self {
        self.cancel = cancel;
        self
    }

    /// Set inline style for the action button
    pub fn with_action_button_style(mut self, style: Option<BTreeMap<String, String>>) -> Self {
        self.action_button_style = style;
        self
    }

    /// Set inline style for the cancel button
    pub fn with_cancel_button_style(mut self, style: Option<BTreeMap<String, String>>) -> Self {
        self.cancel_button_style = style;
        self
    }

    /// Set the ARIA label for the close button
    pub fn with_close_button_aria_label<S: Into<String>>(mut self, label: Option<S>) -> Self {
        self.close_button_aria_label = label.map(Into::into);
        self
    }

    /// Set on_auto_close callback
    pub fn with_on_auto_close(mut self, cb: Option<Callback<u64>>) -> Self {
        self.on_auto_close = cb;
        self
    }

    /// Set on_dismiss callback
    pub fn with_on_dismiss(mut self, cb: Option<Callback<u64>>) -> Self {
        self.on_dismiss = cb;
        self
    }
}

/// Configuration for promise-based toasts (Phase 9)
#[derive(Clone, Debug)]
pub struct PromiseConfig {
    pub loading: String,
    pub success: String,
    pub error: String,
}

impl PromiseConfig {
    pub fn new<L: Into<String>, S: Into<String>, E: Into<String>>(
        loading: L,
        success: S,
        error: E,
    ) -> Self {
        Self {
            loading: loading.into(),
            success: success.into(),
            error: error.into(),
        }
    }
}

/// Internal runtime representation of a toast (subset for Phase 1)
#[derive(Clone)]
pub struct ToastT {
    pub id: u64,
    pub toaster_id: Option<String>,
    pub title: Option<String>,
    pub toast_type: ToastType,
    pub icon: Option<String>, // symbolic icon id for Phase 1
    pub description: Option<String>,
    pub duration_ms: Option<u64>,
    pub delete: bool,
    pub close_button: bool,
    pub dismissible: bool,
    pub action: Option<Action>,
    pub cancel: Option<Action>,
    pub class_name: Option<String>,
    pub class_names: Option<ToastClassNames>,
    pub position: Position,
    pub test_id: Option<String>,
    pub on_auto_close: Option<Callback<u64>>, // Phase 3: notify when a toast auto-closes
    pub on_dismiss: Option<Callback<u64>>,    // Phase 9+: notify when a toast is dismissed/removed
}

impl Default for ToastT {
    fn default() -> Self {
        Self {
            id: 0,
            toaster_id: None,
            title: None,
            toast_type: ToastType::Info,
            icon: None,
            description: None,
            duration_ms: Some(DEFAULT_TOAST_LIFETIME_MS),
            delete: false,
            close_button: false,
            dismissible: true,
            action: None,
            cancel: None,
            class_name: None,
            class_names: None,
            position: Position::BottomRight,
            test_id: None,
            on_auto_close: None,
            on_dismiss: None,
        }
    }
}

/// Toaster-level props (provider defaults)
#[derive(Clone, Debug)]
pub struct ToasterProps {
    pub id: Option<String>,
    pub invert: bool,
    pub theme: Theme,
    pub position: Position,
    pub hotkey: Option<Vec<String>>, // captured as strings like "Alt+T" for Phase 1
    pub rich_colors: bool,
    pub expand: bool,
    pub duration_ms: u64,
    pub gap_px: i32,
    pub visible_toasts: usize,
    pub close_button: bool,
    pub toast_options: ToastOptions,
    pub class_name: Option<String>,
    pub style: Option<BTreeMap<String, String>>,
    pub offset: Offset,
    pub mobile_offset: Offset,
    pub mobile_breakpoint_px: i32,
    pub dir: TextDirection,
    pub swipe_directions: Vec<SwipeDirection>,
    pub icons: ToastIcons,
    pub container_aria_label: Option<String>,
}

impl Default for ToasterProps {
    fn default() -> Self {
        Self {
            id: None,
            invert: false,
            theme: Theme::System,
            position: Position::BottomRight,
            hotkey: None,
            rich_colors: false,
            expand: false,
            duration_ms: DEFAULT_TOAST_LIFETIME_MS,
            gap_px: DEFAULT_GAP_PX,
            visible_toasts: DEFAULT_VISIBLE_TOASTS,
            close_button: false,
            toast_options: ToastOptions::default(),
            class_name: None,
            style: None,
            offset: Offset::Text(DEFAULT_VIEWPORT_OFFSET.to_string()),
            mobile_offset: Offset::Text(DEFAULT_MOBILE_VIEWPORT_OFFSET.to_string()),
            mobile_breakpoint_px: 640,
            dir: TextDirection::Auto,
            swipe_directions: vec![],
            icons: ToastIcons::default(),
            container_aria_label: None,
        }
    }
}

/// Theme preference
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

/// Text direction
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextDirection {
    Rtl,
    Ltr,
    Auto,
}

/// Height measurement record for stacking calculations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeightT {
    pub height_px: i32,
    pub toast_id: u64,
    pub position: Position,
}

// Defaults aligned with Sonner reference
pub const DEFAULT_VISIBLE_TOASTS: usize = 3;
pub const DEFAULT_VIEWPORT_OFFSET: &str = "24px";
pub const DEFAULT_MOBILE_VIEWPORT_OFFSET: &str = "16px";
pub const DEFAULT_TOAST_LIFETIME_MS: u64 = 4000;
pub const DEFAULT_GAP_PX: i32 = 14;
pub const DEFAULT_SWIPE_THRESHOLD_PX: i32 = 45;
