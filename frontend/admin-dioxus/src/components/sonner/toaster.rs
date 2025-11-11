//! Sonner Toaster Provider — Phase 2 (basic render, no animations)

use crate::components::portal_v2::{use_portal, PortalIn, PortalOut};
use dioxus::prelude::*;
// use dioxus::logger::tracing;
use dioxus_time::sleep;
use std::collections::VecDeque;
use std::time::Duration;

use super::state::SonnerCtx;
use super::toast::SonnerToast;
use super::types::{
    HeightT, Offset, Position, TextDirection, ToastOptions, ToastT, ToastType, ToasterProps,
    DEFAULT_VIEWPORT_OFFSET,
};

#[derive(Props, Clone)]
pub struct SonnerToasterProps {
    #[props(default = ToasterProps::default())]
    pub defaults: ToasterProps,
    #[props(default = None)]
    pub children: Option<Element>,
}

impl PartialEq for SonnerToasterProps {
    fn eq(&self, _other: &Self) -> bool {
        // Force re-render checks to consider props changed. This avoids requiring
        // ToasterProps (which contains callbacks) to implement PartialEq.
        false
    }
}

#[component]
pub fn SonnerToaster(props: SonnerToasterProps) -> Element {
    // Toasts state (Phase 2: simple list, no stacking/measurements yet)
    let toasts = use_signal(|| VecDeque::<ToastT>::new());
    let heights = use_signal(|| Vec::<HeightT>::new());
    let mut interacting = use_signal(|| false);
    let hidden = use_signal(|| false);

    // Create callbacks for context
    let add_toast = {
        let mut toasts = toasts.clone();
        let default_duration = props.defaults.duration_ms;
        use_callback(move |mut toast: ToastT| {
            // Only apply default duration for non-loading toasts.
            // Loading toasts without duration should remain until resolved.
            if toast.duration_ms.is_none() && toast.toast_type != ToastType::Loading {
                toast.duration_ms = Some(default_duration);
            }
            let mut list = toasts.write();
            list.push_back(toast);
        })
    };

    let update_with_options = {
        let mut toasts = toasts.clone();
        let default_duration = props.defaults.duration_ms;
        use_callback(
            move |(id, title, toast_type, options): (
                u64,
                Option<String>,
                Option<ToastType>,
                ToastOptions,
            )| {
                let mut list = toasts.write();
                if let Some(pos) = list.iter().position(|t| t.id == id) {
                    let mut t = list[pos].clone();
                    if let Some(tt) = toast_type {
                        t.toast_type = tt;
                    }
                    if let Some(new_title) = title {
                        t.title = Some(new_title);
                    }
                    // Merge options: only override when provided
                    if let Some(v) = options.icon {
                        t.icon = Some(v);
                    }
                    if let Some(v) = options.class_name {
                        t.class_name = Some(v);
                    }
                    if let Some(v) = options.class_names {
                        t.class_names = Some(v);
                    }
                    if let Some(v) = options.toaster_id {
                        t.toaster_id = Some(v);
                    }
                    if let Some(v) = options.action {
                        t.action = Some(v);
                    }
                    if let Some(v) = options.cancel {
                        t.cancel = Some(v);
                    }
                    if let Some(v) = options.duration_ms {
                        t.duration_ms = Some(v);
                    }
                    if let Some(v) = options.on_auto_close {
                        t.on_auto_close = Some(v);
                    }
                    if let Some(v) = options.on_dismiss {
                        t.on_dismiss = Some(v);
                    }
                    if let Some(v) = options.close_button {
                        t.close_button = v;
                    }
                    // Apply default duration if transitioning to non-loading without a duration
                    if t.duration_ms.is_none() && t.toast_type != ToastType::Loading {
                        t.duration_ms = Some(default_duration);
                    }
                    list[pos] = t;
                }
            },
        )
    };

    let update_toast = {
        let mut toasts = toasts.clone();
        let default_duration = props.defaults.duration_ms;
        use_callback(move |mut toast: ToastT| {
            // If an updated toast has no duration and is no longer loading, apply default duration.
            // This covers promise-based transitions from Loading -> Success/Error without a set duration.
            if toast.duration_ms.is_none() && toast.toast_type != ToastType::Loading {
                toast.duration_ms = Some(default_duration);
            }
            let mut list = toasts.write();
            if let Some(pos) = list.iter().position(|t| t.id == toast.id) {
                list[pos] = toast;
            }
        })
    };

    let dismiss_toast = {
        let mut toasts = toasts.clone();
        let heights_sig = heights.clone();
        use_callback(move |id: u64| {
            // mark toast as exiting and schedule removal after exit animation
            {
                let mut list = toasts.write();
                if let Some(pos) = list.iter().position(|t| t.id == id) {
                    if !list[pos].delete {
                        list[pos].delete = true;
                    } else {
                        // already exiting; do not schedule again
                        return;
                    }
                } else {
                    return;
                }
            }
            // schedule actual removal and callback after animation time
            let mut toasts_rm = toasts.clone();
            let mut heights_rm = heights_sig.clone();
            spawn(async move {
                // keep in sync with exit transition in SonnerToast (220ms) + small buffer
                sleep(Duration::from_millis(240)).await;
                let cb = {
                    let mut list = toasts_rm.write();
                    if let Some(pos) = list.iter().position(|t| t.id == id) {
                        let cb = list.get(pos).and_then(|t| t.on_dismiss.clone());
                        list.remove(pos);
                        cb
                    } else {
                        None
                    }
                };
                // remove any recorded height for this toast id
                let mut hs = heights_rm.write();
                if let Some(pos) = hs.iter().position(|h| h.toast_id == id) {
                    hs.remove(pos);
                }
                if let Some(cb) = cb {
                    cb.call(id);
                }
            });
        })
    };

    let delete_toast = {
        let mut toasts = toasts.clone();
        let mut heights_sig = heights.clone();
        use_callback(move |id: u64| {
            let mut list = toasts.write();
            let cb = if let Some(pos) = list.iter().position(|t| t.id == id) {
                list.get(pos).and_then(|t| t.on_dismiss.clone())
            } else {
                None
            };
            if let Some(pos) = list.iter().position(|t| t.id == id) {
                list.remove(pos);
            }
            // remove any recorded height for this toast id
            let mut hs = heights_sig.write();
            if let Some(pos) = hs.iter().position(|h| h.toast_id == id) {
                hs.remove(pos);
            }
            if let Some(cb) = cb {
                cb.call(id);
            }
        })
    };

    use_context_provider(|| SonnerCtx {
        // signals/defaults
        toasts: toasts.clone(),
        heights: heights.clone(),
        interacting: interacting.clone(),
        hidden: hidden.clone(),
        defaults: props.defaults.clone(),
        // callbacks
        add_toast,
        update_toast,
        update_with_options,
        dismiss_toast: dismiss_toast.clone(),
        delete_toast,
    });

    let portal = use_portal();

    // Track viewport width to choose desktop vs mobile offsets (Phase 5)
    let viewport_width = use_signal(|| 1024i32);
    use_effect(move || {
        let mut eval = dioxus::document::eval(
            "(function(){ const send=()=>dioxus.send(window.innerWidth); send(); window.addEventListener('resize', send); })()",
        );
        let mut vw_sig = viewport_width.clone();
        spawn(async move {
            while let Ok(w) = eval.recv::<i32>().await {
                vw_sig.set(w);
            }
        });
    });

    // Listen for document visibility changes to pause timers when hidden
    use_effect(move || {
        let mut eval = dioxus::document::eval(
            "document.addEventListener('visibilitychange', () => { dioxus.send(document.hidden) })",
        );
        let mut hidden_sig = hidden.clone();
        spawn(async move {
            while let Ok(flag) = eval.recv().await {
                hidden_sig.set(flag);
            }
        });
    });

    // Compute container attributes and stacking offsets
    let toasts_vec = {
        let list = toasts.read();
        list.iter().cloned().collect::<Vec<_>>()
    };
    let count = toasts_vec.len();
    let container_label = props
        .defaults
        .container_aria_label
        .clone()
        .unwrap_or_else(|| format!("{} notifications", count));

    // Position -> simple class name for now
    let position_class = match props.defaults.position {
        Position::TopLeft => "sonner-top-left",
        Position::TopRight => "sonner-top-right",
        Position::BottomLeft => "sonner-bottom-left",
        Position::BottomRight => "sonner-bottom-right",
        Position::TopCenter => "sonner-top-center",
        Position::BottomCenter => "sonner-bottom-center",
    };

    let dir_attr = match props.defaults.dir {
        TextDirection::Ltr => "ltr",
        TextDirection::Rtl => "rtl",
        TextDirection::Auto => "auto",
    };

    // Stacking calculations (Phase 4): absolute-position each toast with offsets
    let gap = props.defaults.gap_px.max(0) as i32;
    let fallback_h = 64i32; // px estimate if not yet measured

    let heights_vec = heights.read().clone();
    let heights_px: Vec<i32> = toasts_vec
        .iter()
        .map(|t| {
            heights_vec
                .iter()
                .find(|r| r.toast_id == t.id)
                .map(|r| r.height_px)
                .unwrap_or(fallback_h)
        })
        .collect();

    let mut offsets: Vec<i32> = Vec::with_capacity(toasts_vec.len());
    match props.defaults.position {
        // For top positions, compute cumulative distance from the top for each toast
        Position::TopLeft | Position::TopRight | Position::TopCenter => {
            let mut cursor = 0i32;
            for h in &heights_px {
                offsets.push(cursor);
                cursor += *h + gap;
            }
        }
        // For bottom positions, compute distances from the bottom edge such that the newest toast has 0
        Position::BottomLeft | Position::BottomRight | Position::BottomCenter => {
            let mut cursor = 0i32;
            let mut tmp: Vec<i32> = Vec::with_capacity(toasts_vec.len());
            for h in heights_px.iter().rev() {
                tmp.push(cursor);
                cursor += *h + gap;
            }
            tmp.reverse();
            offsets = tmp;
        }
    }

    // Visible count handling (expand = show all)
    let requested_visible = props.defaults.visible_toasts.max(1);
    let max_visible = if props.defaults.expand {
        usize::MAX
    } else {
        requested_visible
    };
    let visible_count = count.min(max_visible);

    // Container height from visible toasts only
    let visible_height = if visible_count == 0 {
        0
    } else {
        match props.defaults.position {
            Position::TopLeft | Position::TopRight | Position::TopCenter => {
                let start = count - visible_count;
                let slice = &heights_px[start..count];
                let sum: i32 = slice.iter().sum();
                sum + gap * ((visible_count as i32) - 1)
            }
            Position::BottomLeft | Position::BottomRight | Position::BottomCenter => {
                let start = count - visible_count;
                let slice = &heights_px[start..count];
                let sum: i32 = slice.iter().sum();
                sum + gap * ((visible_count as i32) - 1)
            }
        }
    };

    // Phase 5: Offsets per position (desktop/mobile)
    let is_mobile = *viewport_width.read() <= props.defaults.mobile_breakpoint_px;
    let active_offset: &Offset = if is_mobile {
        &props.defaults.mobile_offset
    } else {
        &props.defaults.offset
    };

    // Resolve per-side offset with sensible fallback to DEFAULT_VIEWPORT_OFFSET
    let resolve = |off: &Offset, side: &str| -> String {
        match off {
            Offset::Number(n) => format!("{}px", n),
            Offset::Text(s) => s.clone(),
            Offset::Sides {
                top,
                right,
                bottom,
                left,
            } => match side {
                "top" => top
                    .clone()
                    .unwrap_or_else(|| DEFAULT_VIEWPORT_OFFSET.to_string()),
                "right" => right
                    .clone()
                    .unwrap_or_else(|| DEFAULT_VIEWPORT_OFFSET.to_string()),
                "bottom" => bottom
                    .clone()
                    .unwrap_or_else(|| DEFAULT_VIEWPORT_OFFSET.to_string()),
                "left" => left
                    .clone()
                    .unwrap_or_else(|| DEFAULT_VIEWPORT_OFFSET.to_string()),
                _ => DEFAULT_VIEWPORT_OFFSET.to_string(),
            },
        }
    };

    let pos_css = match props.defaults.position {
        Position::TopLeft => format!(
            "top: {}; left: {};",
            resolve(active_offset, "top"),
            resolve(active_offset, "left")
        ),
        Position::TopRight => format!(
            "top: {}; right: {};",
            resolve(active_offset, "top"),
            resolve(active_offset, "right")
        ),
        Position::BottomLeft => format!(
            "bottom: {}; left: {};",
            resolve(active_offset, "bottom"),
            resolve(active_offset, "left")
        ),
        Position::BottomRight => format!(
            "bottom: {}; right: {};",
            resolve(active_offset, "bottom"),
            resolve(active_offset, "right")
        ),
        // For center positions, let the toasts self-center; container just spans inline.
        Position::TopCenter => {
            format!("top: {}; left: 0; right: 0;", resolve(active_offset, "top"))
        }
        Position::BottomCenter => format!(
            "bottom: {}; left: 0; right: 0;",
            resolve(active_offset, "bottom")
        ),
    };

    let container_style = format!(
        "position: fixed; {} height: {}px;",
        pos_css,
        visible_height.max(0)
    );

    rsx! {
        {props.children}

        PortalIn { portal,
            // Container region
            div {
                role: "region",
                aria_label: "{container_label}",
                tabindex: "-1",
                dir: "{dir_attr}",
                class: "sonner-container {position_class}",
                onmouseenter: move |_| interacting.set(true),
                onmouseleave: move |_| interacting.set(false),
                style: container_style,

                // Visible list with stacking/offsets
                for (i, toast) in toasts_vec.iter().enumerate() {
                    SonnerToast {
                        key: "{toast.id}",
                        id: toast.id,
                        title: toast.title.clone(),
                        description: toast.description.clone(),
                        toast_type: toast.toast_type,
                        icon: toast.icon.clone(),
                        close_button: toast.close_button,
                        exiting: toast.delete,
                        duration_ms: toast.duration_ms,
                        on_auto_close: toast.on_auto_close.clone(),
                        action: toast.action.clone(),
                        cancel: toast.cancel.clone(),
                        layout_css: {Some(match props.defaults.position {
                            // Top cluster (mirror bottom semantics: show the last N toasts)
                            Position::TopLeft | Position::TopRight | Position::TopCenter => {
                                let h_align = match props.defaults.position {
                                    Position::TopLeft => "left:0;",
                                    Position::TopRight => "right:0;",
                                    Position::TopCenter => "left:50%;",
                                    _ => "",
                                };
                                if visible_count == 0 {
                                    format!("position:absolute; {} top:0px; pointer-events: none; z-index:{}; will-change: transform, opacity, top;", h_align, 1000)
                                } else {
                                    let visible_start = count - visible_count;
                                    if i >= visible_start {
                                        // Visible region: newest at top (top:0), older pushed down
                                        let base_top = offsets.get(visible_start).cloned().unwrap_or(0);
                                        let dist_from_slice_top = offsets.get(i).cloned().unwrap_or(0) - base_top;
                                        let h_i = heights_px.get(i).cloned().unwrap_or(fallback_h);
                                        let top_px = (visible_height - (dist_from_slice_top + h_i)).max(0);
                                        let z = 1000 - (count - i) as i32;
                                        // tracing::info!(
                                        //     "Sonner top render: id={}, i={}, z-index={}, height_px={}, top={}px, pos={:?}",
                                        //     toast.id, i, z, h_i, top_px, props.defaults.position
                                        // );
                                        format!(
                                            "position:absolute; {} top:{}px; pointer-events: auto; z-index:{}; will-change: transform, opacity, top;",
                                            h_align,
                                            top_px,
                                            z
                                        )
                                    } else {
                                        // Overflow above the visible cluster (older toasts)
                                        let base_top = 0; // stack at top edge
                                        let z = 1000 - (count - visible_start) as i32;
                                        format!(
                                            "position:absolute; {} top:{}px; pointer-events: none; z-index:{}; will-change: transform, opacity, top;",
                                            h_align,
                                            base_top,
                                            z
                                        )
                                    }
                                }
                            }
                            // Bottom cluster
                            Position::BottomLeft | Position::BottomRight | Position::BottomCenter => {
                                let h_align = match props.defaults.position {
                                    Position::BottomLeft => "left:0;",
                                    Position::BottomRight => "right:0;",
                                    Position::BottomCenter => "left:50%;",
                                    _ => "",
                                };
                                if visible_count == 0 {
                                    format!("position:absolute; {} bottom:0px; pointer-events: none; z-index:{}; will-change: transform, opacity, bottom;", h_align, 1000)
                                } else {
                                    let visible_start = count - visible_count;
                                    if i >= visible_start {
                                        let bottom_px = offsets.get(i).cloned().unwrap_or(0);
                                        let z = 1000 + (count - i) as i32;
                                        // let h_i = heights_px.get(i).cloned().unwrap_or(fallback_h);
                                        // tracing::info!(
                                        //     "Sonner bottom render: id={}, i={}, z-index={}, height_px={}, bottom={}px, pos={:?}",
                                        //     toast.id, i, z, h_i, bottom_px, props.defaults.position
                                        // );
                                        format!(
                                            "position:absolute; {} bottom:{}px; pointer-events: auto; z-index:{}; will-change: transform, opacity, bottom;",
                                            h_align,
                                            bottom_px,
                                            z
                                        )
                                    } else {
                                        let base_bottom = offsets.get(visible_start).cloned().unwrap_or(0);
                                        let z = 1000 - (count - visible_start) as i32;
                                        format!(
                                            "position:absolute; {} bottom:{}px; pointer-events: none; z-index:{}; will-change: transform, opacity, bottom;",
                                            h_align,
                                            base_bottom,
                                            z
                                        )
                                    }
                                }
                            }
                        })},
                        base_transform: {match props.defaults.position {
                            Position::TopLeft | Position::TopRight | Position::TopCenter => {
                                let visible_start = count.saturating_sub(visible_count);
                                if visible_count > 0 && i < visible_start {
                                    // Overflow above visible cluster
                                    let overflow_index = visible_start - i; // 1,2,..
                                    let scale = (1.0 - (overflow_index as f32) * 0.06).max(0.82);
                                    Some(format!("scale({:.3})", scale))
                                } else { None }
                            }
                            Position::BottomLeft | Position::BottomRight | Position::BottomCenter => {
                                let visible_start = count.saturating_sub(visible_count);
                                if visible_count > 0 && i < visible_start {
                                    let overflow_index = visible_start - i;
                                    let scale = (1.0 - (overflow_index as f32) * 0.06).max(0.82);
                                    Some(format!("scale({:.3})", scale))
                                } else { None }
                            }
                        }},
                        on_close: {
                            let dismiss_toast = dismiss_toast.clone();
                            let id = toast.id;
                            Callback::new(move |_| dismiss_toast.call(id))
                        },
                    }
                }
            }
        }
        PortalOut { portal }
    }
}
