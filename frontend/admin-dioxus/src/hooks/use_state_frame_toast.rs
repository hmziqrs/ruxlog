use crate::components::sonner::{use_sonner, ToastOptions};
use crate::store::StateFrame;
use dioxus::prelude::*;
use std::collections::HashMap;
use std::hash::Hash;

#[derive(Clone)]
pub struct StateFrameToastConfig {
    pub loading_title: String,
    pub success_title: Option<String>,
    pub error_title: Option<String>,
    pub loading_options: ToastOptions,
    pub success_options: ToastOptions,
    pub error_options: ToastOptions,
}

impl Default for StateFrameToastConfig {
    fn default() -> Self {
        Self {
            loading_title: "Processing...".to_string(),
            success_title: None,
            error_title: None,
            // Keep loading indefinite; provider will set duration on settle if None
            loading_options: ToastOptions::default().with_duration(None),
            // Let provider default duration apply on settle unless caller overrides
            success_options: ToastOptions::default(),
            error_options: ToastOptions::default(),
        }
    }
}

/// Wire a StateFrame<T> to Sonner toasts.
/// Shows/updates a loading toast when entering Loading, and updates to Success/Error when leaving Loading.
pub fn use_state_frame_toast<D: Clone + 'static, M: Clone + 'static>(
    frame: &GlobalSignal<StateFrame<D, M>>,
    cfg: StateFrameToastConfig,
) {
    let sonner = use_sonner();
    let mut toast_id = use_signal::<Option<u64>>(|| None);

    // Read current status flags once per render
    let state = frame.read();
    let loading = state.is_loading();
    let success = state.is_success();
    let failed = state.is_failed();
    let error_message = state.error_message();

    // Track previous loading to detect edges
    let prev_loading = crate::hooks::use_previous(loading);

    use_effect(use_reactive!(|(loading, success, failed)| {
        if let Some(prev) = prev_loading {
            // Entering loading
            if !prev && loading {
                match toast_id() {
                    Some(id) if sonner.exists(id) => {
                        sonner.update_loading(
                            id,
                            cfg.loading_title.clone(),
                            cfg.loading_options.clone(),
                        );
                    }
                    _ => {
                        let id =
                            sonner.loading(cfg.loading_title.clone(), cfg.loading_options.clone());
                        toast_id.set(Some(id));
                    }
                }
            }

            // Leaving loading: settle
            if prev && !loading {
                if let Some(id) = toast_id() {
                    if success {
                        let title = cfg
                            .success_title
                            .clone()
                            .unwrap_or_else(|| "Operation completed".to_string());
                        sonner.update_success(id, title, cfg.success_options.clone());
                    } else if failed {
                        let title = cfg
                            .error_title
                            .clone()
                            .or_else(|| error_message.clone())
                            .unwrap_or_else(|| "Operation failed".to_string());
                        sonner.update_error(id, title, cfg.error_options.clone());
                    }
                }
            }
        }
    }));
}

/// Wire a keyed `StateFrame` (found in a `HashMap`) to Sonner toasts.
/// Allows reuse of the toast side-effects for edit/remove flows that track state per id.
pub fn use_state_frame_map_toast<K, D, M>(
    map: &GlobalSignal<HashMap<K, StateFrame<D, M>>>,
    id: K,
    cfg: StateFrameToastConfig,
) where
    K: Eq + Hash + Copy + 'static,
    D: Clone + 'static,
    M: Clone + 'static,
{
    let sonner = use_sonner();
    let mut toast_id = use_signal::<Option<u64>>(|| None);

    let frame = map.read().get(&id).cloned().unwrap_or_else(StateFrame::new);
    let loading = frame.is_loading();
    let success = frame.is_success();
    let failed = frame.is_failed();
    let error_message = frame.error_message();

    let prev_loading = crate::hooks::use_previous(loading);

    use_effect(use_reactive!(|(loading, success, failed)| {
        if let Some(prev) = prev_loading {
            if !prev && loading {
                match toast_id() {
                    Some(id) if sonner.exists(id) => {
                        sonner.update_loading(
                            id,
                            cfg.loading_title.clone(),
                            cfg.loading_options.clone(),
                        );
                    }
                    _ => {
                        let id =
                            sonner.loading(cfg.loading_title.clone(), cfg.loading_options.clone());
                        toast_id.set(Some(id));
                    }
                }
            }

            if prev && !loading {
                if let Some(id) = toast_id() {
                    if success {
                        let title = cfg
                            .success_title
                            .clone()
                            .unwrap_or_else(|| "Operation completed".to_string());
                        sonner.update_success(id, title, cfg.success_options.clone());
                    } else if failed {
                        let title = cfg
                            .error_title
                            .clone()
                            .or_else(|| error_message.clone())
                            .unwrap_or_else(|| "Operation failed".to_string());
                        sonner.update_error(id, title, cfg.error_options.clone());
                    }
                }
            }
        }
    }));
}
