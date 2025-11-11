use dioxus::prelude::*;
use dioxus_time::sleep;
use std::time::Duration;

use super::*;
use crate::components::animated_grid::provider::use_grid_context;

#[component]
pub fn AnimatedGridCircle(circle: CircleSignal) -> Element {
    let grid_ctx = use_grid_context();
    let grid_ctx_clone = grid_ctx.clone();

    // Track if we've already triggered the initial spawn
    let mut spawn_triggered = use_signal(|| false);

    // Trigger initial scale-in animation when circle is first created
    use_effect(move || {
        if *spawn_triggered.read() {
            return; // Already triggered
        }
        
        let is_respawning = circle.read().respawning;
        if !is_respawning {
            return;
        }
        
        spawn_triggered.set(true);
        
        spawn({
            let circle_sig = circle;
            let grid_ctx = grid_ctx_clone.clone();
            async move {
                // Random delay between 100ms and 1000ms before spawning
                let delay = 100 + (random_u64() % 1600);
                sleep(Duration::from_millis(delay)).await;
                schedule_post_respawn(circle_sig, grid_ctx);
            }
        });
    });

    let (x, y, circle_state, is_scaling_in) = {
        let state = circle.read();
        let grid = grid_ctx.grid_data.read();

        let Some((x, y)) = indices_to_px(state.col, state.row, &grid) else {
            return rsx! {};
        };
        let is_scaling_in = state.is_scaling_in_active(&grid);
        (x, y, state.clone(), is_scaling_in)
    };

    let style = if circle_state.is_respawning() {
        // web_sys::console::log_1(
        //     &format!(
        //         "Circle {} RESPAWNING: scale={:.2}, opacity={:.2}, transition=NONE [respawning=true, scaling_in={}, moving={}]",
        //         circle_state.id, circle_state.scale, circle_state.opacity, circle_state.scaling_in, circle_state.moving
        //     )
        //     .into(),
        // );
        format!(
            "transform: translate({x:.2}px, {y:.2}px) scale({:.2}); width: {DIAMETER_PX}px; height: {DIAMETER_PX}px; border-radius: 9999px; opacity: {:.2}; transition: none;",
            circle_state.scale, circle_state.opacity
        )
    } else if is_scaling_in || circle_state.is_scaling_out_active() {
        // let phase = if is_scaling_in { "SCALING_IN" } else { "SCALING_OUT" };
        // web_sys::console::log_1(
        //     &format!(
        //         "Circle {} {}: scale={:.2}, opacity={:.2}, duration={}ms [respawning={}, scaling_in={}, moving={}]",
        //         circle_state.id, phase, circle_state.scale, circle_state.opacity, SCALE_DURATION_MS,
        //         circle_state.respawning, circle_state.scaling_in, circle_state.moving
        //     )
        //     .into(),
        // );
        format!(
            "transform: translate({x:.2}px, {y:.2}px) scale({:.2}); width: {DIAMETER_PX}px; height: {DIAMETER_PX}px; border-radius: 9999px; opacity: {:.2}; transition: transform {SCALE_DURATION_MS}ms ease-in, opacity {SCALE_DURATION_MS}ms ease-in;",
            circle_state.scale, circle_state.opacity
        )
    } else {
        // web_sys::console::log_1(
        //     &format!(
        //         "Circle {} MOVING: scale={:.2}, opacity={:.2}, duration={}ms [respawning={}, scaling_in={}, moving={}]",
        //         circle_state.id, circle_state.scale, circle_state.opacity, STEP_DURATION_MS,
        //         circle_state.respawning, circle_state.scaling_in, circle_state.moving
        //     )
        //     .into(),
        // );
        format!(
            "transform: translate({x:.2}px, {y:.2}px) scale({:.2}); width: {DIAMETER_PX}px; height: {DIAMETER_PX}px; border-radius: 9999px; opacity: {:.2}; transition: transform {STEP_DURATION_MS}ms linear;",
            circle_state.scale, circle_state.opacity
        )
    };

    // info!("state: {:?}", circle_state);

    rsx! {
        div {
            class: "absolute will-change-transform pointer-events-none bg-primary shadow-[0_0_8px_1px_var(--primary)]",
            style: "{style}",
            ontransitionend: move |_| {
                handle_transition_end(circle, grid_ctx.clone());
            },
        }
    }
}
