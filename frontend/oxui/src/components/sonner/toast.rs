//! Sonner Single Toast View — Phase 2 minimal view + Phase 7 swipe/drag to dismiss

use crate::hooks::use_unique_id;
use dioxus::prelude::*;
use dioxus_time::sleep;
use std::time::Duration;

use super::icons::{icon_close, icon_error, icon_info, icon_success, icon_warning, loader_spinner};
use super::state::SonnerCtx;
use super::types::{
    Action, HeightT, Position, SwipeDirection, ToastType, DEFAULT_SWIPE_THRESHOLD_PX,
};

#[derive(Props, Clone)]
pub struct SonnerToastProps {
    pub id: u64,
    pub title: Option<String>,
    pub description: Option<String>,
    pub toast_type: ToastType,
    #[props(default = None)]
    pub icon: Option<String>,
    #[props(default = false)]
    pub close_button: bool,
    /// When true, play exit animation (fade + slide) and expect provider to remove after delay
    #[props(default = false)]
    pub exiting: bool,
    #[props(default = None)]
    pub duration_ms: Option<u64>,
    #[props(default = None)]
    pub on_auto_close: Option<Callback<u64>>,
    #[props(default = None)]
    pub action: Option<Action>,
    #[props(default = None)]
    pub cancel: Option<Action>,
    /// Provider-computed layout CSS (position/top|bottom/z-index/left|right/pointer-events/will-change)
    /// IMPORTANT: Should NOT include `transform` or `transition`.
    #[props(default = None)]
    pub layout_css: Option<String>,
    /// Optional base transform to compose before live drag (e.g., scale for overflow stacking)
    /// Example: Some("scale(0.88)")
    #[props(default = None)]
    pub base_transform: Option<String>,
    pub on_close: Callback<MouseEvent>,
    #[props(extends = GlobalAttributes)]
    attributes: Vec<Attribute>,
}

impl PartialEq for SonnerToastProps {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.title == other.title
            && self.description == other.description
            && self.toast_type == other.toast_type
            && self.icon == other.icon
            && self.close_button == other.close_button
            && self.exiting == other.exiting
            && self.duration_ms == other.duration_ms
            && self.layout_css == other.layout_css
            && self.base_transform == other.base_transform
        // Intentionally ignore action/cancel/on_close/attributes due to callbacks and dynamic bags
    }
}

#[component]
pub fn SonnerToast(props: SonnerToastProps) -> Element {
    let ctx = use_context::<SonnerCtx>();
    let uid = use_unique_id();
    let id = use_memo(move || format!("sonner-toast-{uid}"));
    let label_id = format!("{id}-label");
    let description_id = props
        .description
        .as_ref()
        .map(|_| format!("{id}-description"));
    let aria_labelledby_val = label_id.clone();
    let aria_describedby_val = description_id.clone();

    // Precompute action/cancel options and handlers to capture into closures
    let action_opt = props.action.clone();
    let cancel_opt = props.cancel.clone();
    let action_on_click = action_opt.as_ref().and_then(|a| a.on_click.clone());
    let cancel_on_click = cancel_opt.as_ref().and_then(|a| a.on_click.clone());
    let action_dismiss_toast = action_opt.as_ref().map(|a| a.dismiss_toast).unwrap_or(true);

    // Entrance animation: fade + slide in on mount
    let mounted = use_signal(|| false);
    {
        let mounted_read = mounted.clone();
        let mut mounted_set = mounted.clone();
        use_effect(move || {
            if mounted_read() {
                return;
            }
            // Defer to next tick to ensure initial style is applied before transition
            spawn(async move {
                sleep(Duration::from_millis(16)).await;
                mounted_set.set(true);
            });
        });
    }

    // Phase 4: measure height on mount and when content changes
    {
        let heights = ctx.heights.clone();
        let position = ctx.defaults.position;
        let id_for_measure = format!("{id}");
        let toast_id_for_height = props.id;
        let _title_dep = props.title.clone();
        let _desc_dep = props.description.clone();
        use_effect(move || {
            let mut eval = dioxus::document::eval(&format!(
                "(() => {{ const el = document.getElementById('{}'); dioxus.send(el ? Math.round(el.getBoundingClientRect().height) : 0) }})()",
                id_for_measure
            ));
            let mut heights_sig = heights.clone();
            spawn(async move {
                if let Ok(h) = eval.recv::<i32>().await {
                    let mut vec = heights_sig.write();
                    if let Some(existing) =
                        vec.iter_mut().find(|r| r.toast_id == toast_id_for_height)
                    {
                        existing.height_px = h as i32;
                        existing.position = position;
                    } else {
                        vec.push(HeightT {
                            height_px: h as i32,
                            toast_id: toast_id_for_height,
                            position,
                        });
                    }
                }
            });
        });
    }

    // Phase 3: timers with pause on hover/focus/interacting/hidden
    let mut hovered = use_signal(|| false);
    let mut focused = use_signal(|| false);
    let interacting = ctx.interacting;
    let hidden = ctx.hidden;
    let paused = use_memo(move || hovered() || focused() || interacting() || hidden());
    let mut remaining_ms = use_signal(|| props.duration_ms.unwrap_or(0));
    let toast_id_for_timer = props.id;
    let on_auto_close_cb = props.on_auto_close.clone();
    let init_duration = props.duration_ms;
    let dismiss = ctx.dismiss_toast;
    let has_duration = props.duration_ms.is_some();
    let mut started = use_signal(|| false);
    use_effect(move || {
        if has_duration && !started() {
            started.set(true);
            // Initialize remaining time from current props, important for promise-updated toasts
            remaining_ms.set(init_duration.unwrap_or(0));
            spawn(async move {
                let tick = 50u64; // ms
                loop {
                    if paused() {
                        sleep(Duration::from_millis(tick)).await;
                        continue;
                    }
                    let current = remaining_ms();
                    if current == 0 {
                        break;
                    }
                    let next = current.saturating_sub(tick);
                    remaining_ms.set(next);
                    if next == 0 {
                        break;
                    }
                    sleep(Duration::from_millis(tick)).await;
                }
                // Auto close
                dismiss.call(toast_id_for_timer);
                if let Some(cb) = on_auto_close_cb.clone() {
                    cb.call(toast_id_for_timer);
                }
            });
        }
    });

    // Phase 7: swipe/drag to dismiss
    // Determine allowed swipe directions: use defaults if provided, else infer from position
    let allowed_dirs = {
        let cfg = ctx.defaults.swipe_directions.clone();
        if !cfg.is_empty() {
            cfg
        } else {
            match ctx.defaults.position {
                Position::TopLeft | Position::BottomLeft => vec![SwipeDirection::Left],
                Position::TopRight | Position::BottomRight => vec![SwipeDirection::Right],
                Position::TopCenter => vec![SwipeDirection::Top],
                Position::BottomCenter => vec![SwipeDirection::Bottom],
            }
        }
    };

    let drag_dx = use_signal(|| 0.0f64);
    let drag_dy = use_signal(|| 0.0f64);
    let dragging = use_signal(|| false);
    let snapping = use_signal(|| false);

    // Attach JS pointer/touch listeners to track drag deltas
    {
        let _id_for_drag = format!("{id}");
        let mut dx_sig = drag_dx.clone();
        let mut dy_sig = drag_dy.clone();
        let mut dragging_sig = dragging.clone();
        let mut snapping_sig = snapping.clone();
        let dismiss = ctx.dismiss_toast.clone();
        let toast_id = props.id;
        let allowed = allowed_dirs.clone();
        use_effect(move || {
            let script = format!(
                r#"(function() {{
                const el = document.getElementById('{id}');
                if (!el) {{ dioxus.send(['noop',0,0]); return; }}
                if (el.dataset.dragBound === '1') {{ return; }}
                el.dataset.dragBound = '1';
                let startX=0, startY=0, active=false;
                function onDown(ev) {{
                    const p = ev.touches ? ev.touches[0] : ev;
                    startX = p.clientX; startY = p.clientY; active = true;
                    dioxus.send(['start', 0, 0]);
                    window.addEventListener('mousemove', onMove);
                    window.addEventListener('touchmove', onMove, {{ passive: false }});
                    window.addEventListener('mouseup', onUp);
                    window.addEventListener('touchend', onUp);
                    window.addEventListener('touchcancel', onUp);
                }}
                function onMove(ev) {{
                    if (!active) return;
                    const p = ev.touches ? ev.touches[0] : ev;
                    const dx = p.clientX - startX; const dy = p.clientY - startY;
                    dioxus.send(['move', dx, dy]);
                    if (ev.cancelable) ev.preventDefault();
                }}
                function onUp(ev) {{
                    if (!active) return;
                    active = false;
                    const p = (ev.changedTouches && ev.changedTouches[0]) ? ev.changedTouches[0] : ev;
                    const dx = p.clientX - startX; const dy = p.clientY - startY;
                    dioxus.send(['end', dx, dy]);
                    window.removeEventListener('mousemove', onMove);
                    window.removeEventListener('touchmove', onMove);
                    window.removeEventListener('mouseup', onUp);
                    window.removeEventListener('touchend', onUp);
                    window.removeEventListener('touchcancel', onUp);
                }}
                el.addEventListener('mousedown', onDown);
                el.addEventListener('touchstart', onDown, {{ passive: true }});
            }})()"#
            );
            let mut eval = dioxus::document::eval(&script);
            let allowed_for_task = allowed.clone();
            spawn(async move {
                while let Ok((typ, dx, dy)) = eval.recv::<(String, f64, f64)>().await {
                    match typ.as_str() {
                        "start" => {
                            dragging_sig.set(true);
                            snapping_sig.set(false);
                        }
                        "move" => {
                            dx_sig.set(dx);
                            dy_sig.set(dy);
                        }
                        "end" => {
                            dragging_sig.set(false);
                            // Project movement onto allowed axis and decide dismissal with sign check
                            let ax_h = allowed_for_task
                                .iter()
                                .any(|d| matches!(d, SwipeDirection::Left | SwipeDirection::Right));
                            let ax_v = allowed_for_task
                                .iter()
                                .any(|d| matches!(d, SwipeDirection::Top | SwipeDirection::Bottom));
                            let use_h = ax_h && (!ax_v || dx.abs() >= dy.abs());
                            let threshold = DEFAULT_SWIPE_THRESHOLD_PX as f64;
                            let mut should_dismiss = false;
                            if use_h {
                                if dx <= -threshold
                                    && allowed_for_task
                                        .iter()
                                        .any(|d| matches!(d, SwipeDirection::Left))
                                {
                                    should_dismiss = true;
                                }
                                if dx >= threshold
                                    && allowed_for_task
                                        .iter()
                                        .any(|d| matches!(d, SwipeDirection::Right))
                                {
                                    should_dismiss = true;
                                }
                            } else if ax_v {
                                if dy <= -threshold
                                    && allowed_for_task
                                        .iter()
                                        .any(|d| matches!(d, SwipeDirection::Top))
                                {
                                    should_dismiss = true;
                                }
                                if dy >= threshold
                                    && allowed_for_task
                                        .iter()
                                        .any(|d| matches!(d, SwipeDirection::Bottom))
                                {
                                    should_dismiss = true;
                                }
                            }
                            if should_dismiss {
                                // Trigger dismiss; keep deltas to allow a natural fling-out fade
                                dismiss.call(toast_id);
                                // Do not reset dx/dy here; exit animation will fade/slide out from current offset
                            } else {
                                // Snap back with transition
                                snapping_sig.set(true);
                                dx_sig.set(0.0);
                                dy_sig.set(0.0);
                            }
                        }
                        _ => {}
                    }
                }
            });
        });
    }

    // Compute combined drag + entrance style for the OUTER toast element
    let initial_offset = match ctx.defaults.position {
        Position::TopLeft | Position::TopCenter | Position::TopRight => -10.0,
        Position::BottomLeft | Position::BottomCenter | Position::BottomRight => 10.0,
    };
    let dx = drag_dx();
    let dy = drag_dy();
    let ax_h = allowed_dirs
        .iter()
        .any(|d| matches!(d, SwipeDirection::Left | SwipeDirection::Right));
    let ax_v = allowed_dirs
        .iter()
        .any(|d| matches!(d, SwipeDirection::Top | SwipeDirection::Bottom));
    let use_h = ax_h && (!ax_v || dx.abs() >= dy.abs());
    let (tx, ty) = if use_h {
        (dx, 0.0)
    } else if ax_v {
        (0.0, dy)
    } else {
        (0.0, 0.0)
    };
    let ratio =
        ((if use_h { dx.abs() } else { dy.abs() }) / (DEFAULT_SWIPE_THRESHOLD_PX as f64)).min(1.0);
    let drag_opacity = 1.0 - 0.3 * ratio;
    // Entrance/Exit composition: start hidden and offset, animate to rest; on exit fade + slide away
    let enter_transition =
        "transform 220ms ease, opacity 220ms ease, top 200ms ease, bottom 200ms ease";
    let exit_transition =
        "transform 220ms ease, opacity 220ms ease, top 200ms ease, bottom 200ms ease";
    let is_exiting = props.exiting;
    let outer_transition = if dragging() {
        "none"
    } else if snapping() {
        "transform 180ms ease-out, opacity 180ms ease-out"
    } else if !mounted() {
        "none"
    } else if is_exiting {
        exit_transition
    } else {
        enter_transition
    };
    // Offset for enter before mount, and for exit when flagged
    let ty_with_anim = if !mounted() {
        ty + initial_offset
    } else if is_exiting {
        ty + initial_offset
    } else {
        ty
    };
    let outer_opacity = if !mounted() || is_exiting {
        0.0
    } else {
        drag_opacity
    };
    let pe = if is_exiting {
        "pointer-events: none;"
    } else {
        ""
    };
    let is_center = matches!(
        ctx.defaults.position,
        Position::TopCenter | Position::BottomCenter
    );
    let center_prefix = if is_center { "translateX(-50%) " } else { "" };
    let base_tf = props.base_transform.clone().unwrap_or_default();
    let base_tf = if base_tf.is_empty() {
        String::new()
    } else {
        format!("{} ", base_tf)
    };
    let drag_style = format!(
        "transform: {}{}translate({:.2}px, {:.2}px); opacity: {:.3}; transition: {}; {}",
        center_prefix, base_tf, tx, ty_with_anim, outer_opacity, outer_transition, pe
    );

    // Final style = provider layout + our drag/animation style. Put our style last so it wins on conflicts.
    let layout_css = props.layout_css.clone().unwrap_or_default();
    let final_style = format!("{} {}", layout_css, drag_style);

    // Phase 8: resolve icon element based on per-toast and global overrides
    let icon_el: Option<Element> = {
        // Prefer per-toast keyword, else provider defaults by type
        let kw_opt = props.icon.clone().or_else(|| match props.toast_type {
            ToastType::Success => ctx.defaults.icons.success.clone(),
            ToastType::Info => ctx.defaults.icons.info.clone(),
            ToastType::Warning => ctx.defaults.icons.warning.clone(),
            ToastType::Error => ctx.defaults.icons.error.clone(),
            ToastType::Loading => ctx.defaults.icons.loading.clone(),
        });

        if let Some(kw) = kw_opt {
            let k = kw.to_lowercase();
            match k.as_str() {
                "none" => None,
                "success" => Some(icon_success(None)),
                "info" => Some(icon_info(None)),
                "warning" => Some(icon_warning(None)),
                "error" => Some(icon_error(None)),
                "loading" | "loader" => Some(loader_spinner(None)),
                "close" => Some(icon_close(None)),
                _ => {
                    // Fallback to default for this toast type
                    match props.toast_type {
                        ToastType::Success => Some(icon_success(None)),
                        ToastType::Info => Some(icon_info(None)),
                        ToastType::Warning => Some(icon_warning(None)),
                        ToastType::Error => Some(icon_error(None)),
                        ToastType::Loading => Some(loader_spinner(None)),
                    }
                }
            }
        } else {
            // Default for this toast type
            match props.toast_type {
                ToastType::Success => Some(icon_success(None)),
                ToastType::Info => Some(icon_info(None)),
                ToastType::Warning => Some(icon_warning(None)),
                ToastType::Error => Some(icon_error(None)),
                ToastType::Loading => Some(loader_spinner(None)),
            }
        }
    };

    rsx! {
        div {
            id: "{id}",
            role: "alertdialog",
            aria_labelledby: "{aria_labelledby_val}",
            aria_describedby: aria_describedby_val,
            aria_modal: "false",
            tabindex: "0",
            class: "sonner-toast w-72 rounded-md border border-border bg-background text-foreground shadow-sm",
            "data-type": props.toast_type.as_str(),
            // Apply our combined style
            style: "{final_style}",
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| hovered.set(false),
            onfocus: move |_| focused.set(true),
            onblur: move |_| focused.set(false),
            ..props.attributes,

            // Inner wrapper
            div { class: "sonner-toast-inner flex items-center justify-between gap-2 px-4 py-3",
                // Left icon (if any)
                if let Some(icon) = icon_el {
                    div { class: "sonner-toast-icon shrink-0", {icon} }
                }

                div { class: "sonner-toast-content flex-1",
                    role: "alert",
                    aria_atomic: "true",

                    if let Some(title) = &props.title {
                        div { id: label_id.clone(), class: "sonner-toast-title font-semibold", {title.clone()} }
                    }

                    if let Some(description) = &props.description {
                        div { id: description_id.clone(), class: "sonner-toast-description text-sm text-muted-foreground", {description.clone()} }
                    }
                }

                // Action buttons (optional)
                if action_opt.is_some() || cancel_opt.is_some() {
                    div { class: "sonner-toast-actions flex items-center gap-2",
                        if let Some(cancel) = &cancel_opt {
                            button {
                                class: "sonner-toast-cancel px-2 py-1 text-sm rounded border border-border bg-transparent hover:bg-accent text-foreground/80 hover:text-foreground",
                                onclick: move |e| {
                                    if let Some(cb) = cancel_on_click.clone() { cb.call(e); }
                                },
                                {cancel.label.clone()}
                            }
                        }
                        if let Some(action) = &action_opt {
                            button {
                                class: "sonner-toast-action px-2 py-1 text-sm rounded bg-primary text-primary-foreground hover:opacity-90",
                                onclick: move |e| {
                                    if let Some(cb) = action_on_click.clone() { cb.call(e); }
                                    if action_dismiss_toast { ctx.dismiss_toast.call(props.id); }
                                },
                                {action.label.clone()}
                            }
                        }
                    }
                }

                if props.close_button {
                    button {
                        class: "sonner-toast-close self-start p-0 m-0 border-0 bg-transparent leading-none cursor-pointer text-muted-foreground hover:text-foreground",
                        aria_label: "close",
                        onclick: move |e| props.on_close.call(e),
                        { icon_close(None) }
                    }
                }
            }
        }
    }
}
