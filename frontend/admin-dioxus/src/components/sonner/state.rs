//! Sonner (Dioxus) State — Phase 0/1 scaffold
//! Provides handle types and context contracts. Provider implementation comes in Phase 2.

use dioxus::{logger::tracing, prelude::*};
use std::collections::VecDeque;
use std::future::Future;

use super::types::{
    HeightT, Position, PromiseConfig, ToastOptions, ToastT, ToastType, ToasterProps,
    DEFAULT_TOAST_LIFETIME_MS,
};

// Callback types used by the provider (wired in Phase 2)
type AddToastCallback = Callback<ToastT>;
type UpdateToastCallback = Callback<ToastT>;
type UpdateWithOptionsCallback = Callback<(u64, Option<String>, Option<ToastType>, ToastOptions)>;
type DismissToastCallback = Callback<u64>;
type DeleteToastCallback = Callback<u64>;

#[derive(Clone)]
pub struct SonnerCtx {
    // Signals (Phase 2: exposed for future stacking/controls)
    pub toasts: Signal<VecDeque<ToastT>>, // ordered list
    pub heights: Signal<Vec<HeightT>>,    // measurements (unused in Phase 2)
    pub interacting: Signal<bool>,        // pointer within region
    pub hidden: Signal<bool>,             // document.hidden flag (Phase 3)
    pub defaults: ToasterProps,           // effective defaults

    // Imperative controls
    pub add_toast: AddToastCallback,
    pub update_toast: UpdateToastCallback,
    pub update_with_options: UpdateWithOptionsCallback,
    pub dismiss_toast: DismissToastCallback,
    pub delete_toast: DeleteToastCallback,
}

/// Public handle returned by the hook to interact with Sonner provider
#[derive(Clone, Copy)]
pub struct SonnerToasts {
    add_toast: AddToastCallback,
    update_toast: UpdateToastCallback,
    update_with_options: UpdateWithOptionsCallback,
    dismiss_toast: DismissToastCallback,
    delete_toast: DeleteToastCallback,
}

impl SonnerToasts {
    /// Show a toast with given type and options.
    pub fn show(&self, title: String, toast_type: ToastType, options: ToastOptions) -> u64 {
        let id = next_toast_id();
        let toast = ToastT {
            id,
            toaster_id: options.toaster_id.clone(),
            title: Some(title),
            toast_type,
            icon: options.icon.clone(),
            description: None,
            duration_ms: options.duration_ms.or(Some(DEFAULT_TOAST_LIFETIME_MS)),
            delete: false,
            close_button: options.close_button.unwrap_or(false),
            dismissible: true,
            action: options.action.clone(),
            cancel: options.cancel.clone(),
            class_name: options.class_name.clone(),
            class_names: options.class_names.clone(),
            position: Position::BottomRight,
            test_id: None,
            on_auto_close: options.on_auto_close.clone(),
            on_dismiss: options.on_dismiss.clone(),
        };
        self.add_toast.call(toast);
        id
    }

    pub fn success(&self, title: String, options: ToastOptions) -> u64 {
        self.show(title, ToastType::Success, options)
    }
    pub fn error(&self, title: String, options: ToastOptions) -> u64 {
        self.show(title, ToastType::Error, options)
    }
    pub fn warning(&self, title: String, options: ToastOptions) -> u64 {
        self.show(title, ToastType::Warning, options)
    }
    pub fn info(&self, title: String, options: ToastOptions) -> u64 {
        self.show(title, ToastType::Info, options)
    }
    pub fn loading(&self, title: String, options: ToastOptions) -> u64 {
        self.show(title, ToastType::Loading, options)
    }

    pub fn dismiss(&self, id: u64) {
        self.dismiss_toast.call(id)
    }
    pub fn delete(&self, id: u64) {
        self.delete_toast.call(id)
    }

    /// Check if a toast with the given id currently exists in the provider.
    /// Note: this returns true even if the toast is in the exiting state
    /// (i.e., marked for deletion but not yet removed after the exit animation).
    pub fn exists(&self, id: u64) -> bool {
        let ctx = consume_context::<SonnerCtx>();
        let list = ctx.toasts.peek();
        list.iter().any(|t| t.id == id)
    }

    pub fn update(&self, id: u64, mut toast: ToastT) {
        // Ensure the correct target id is used
        toast.id = id;
        self.update_toast.call(toast)
    }

    /// Merge changes into an existing toast by id using ToastOptions,
    /// with optional title/type overrides (e.g., Loading -> Success).
    pub fn update_with_options(
        &self,
        id: u64,
        title: Option<String>,
        toast_type: Option<ToastType>,
        options: ToastOptions,
    ) {
        self.update_with_options
            .call((id, title, toast_type, options))
    }

    /// Convenience: update an existing toast to Success type with a new title and options
    pub fn update_success<T: Into<String>>(&self, id: u64, title: T, options: ToastOptions) {
        self.update_with_options(id, Some(title.into()), Some(ToastType::Success), options)
    }

    /// Convenience: update an existing toast to Error type with a new title and options
    pub fn update_error<T: Into<String>>(&self, id: u64, title: T, options: ToastOptions) {
        self.update_with_options(id, Some(title.into()), Some(ToastType::Error), options)
    }

    /// Alias for update_error for ergonomic "failure" wording
    pub fn update_failure<T: Into<String>>(&self, id: u64, title: T, options: ToastOptions) {
        self.update_error(id, title, options)
    }

    /// Convenience: update an existing toast to Warning type
    pub fn update_warning<T: Into<String>>(&self, id: u64, title: T, options: ToastOptions) {
        self.update_with_options(id, Some(title.into()), Some(ToastType::Warning), options)
    }

    /// Convenience: update an existing toast to Info type
    pub fn update_info<T: Into<String>>(&self, id: u64, title: T, options: ToastOptions) {
        self.update_with_options(id, Some(title.into()), Some(ToastType::Info), options)
    }

    /// Convenience: update an existing toast to Loading type (e.g., resuming a pending state)
    pub fn update_loading<T: Into<String>>(&self, id: u64, title: T, options: ToastOptions) {
        self.update_with_options(id, Some(title.into()), Some(ToastType::Loading), options)
    }

    /// Promise-based toast flow (Phase 9): shows a loading toast, then updates to success or error.
    /// This simplified variant expects a future yielding Result<(), ()> and static messages.
    pub fn promise<F>(&self, fut: F, config: PromiseConfig, options: ToastOptions)
    where
        F: Future<Output = Result<(), ()>> + 'static,
    {
        let id = next_toast_id();

        // Insert loading toast (no duration to keep it until resolution)
        let mut loading = ToastT {
            id,
            toaster_id: options.toaster_id.clone(),
            title: Some(config.loading.clone()),
            toast_type: ToastType::Loading,
            icon: options.icon.clone(),
            description: None,
            duration_ms: None,
            delete: false,
            close_button: options.close_button.unwrap_or(false),
            dismissible: true,
            action: options.action.clone(),
            cancel: options.cancel.clone(),
            class_name: options.class_name.clone(),
            class_names: options.class_names.clone(),
            position: Position::BottomRight,
            test_id: None,
            on_auto_close: options.on_auto_close.clone(),
            on_dismiss: options.on_dismiss.clone(),
        };
        self.add_toast.call(loading.clone());

        let update = self.update_toast;
        spawn(async move {
            let res = fut.await;
            match res {
                Ok(()) => {
                    loading.toast_type = ToastType::Success;
                    loading.title = Some(config.success.clone());
                    // If caller provided a duration in options, use it; else leave None so provider applies its default
                    loading.duration_ms = options.duration_ms;
                }
                Err(()) => {
                    loading.toast_type = ToastType::Error;
                    loading.title = Some(config.error.clone());
                    // If caller provided a duration in options, use it; else leave None so provider applies its default
                    loading.duration_ms = options.duration_ms;
                }
            }
            tracing::info!("Promise result: {:?}", options.duration_ms);
            update.call(loading);
        });
    }
}

/// Hook used by components to obtain a Sonner handle
pub fn use_sonner() -> SonnerToasts {
    use_hook(consume_sonner)
}

/// Consume the provider context and produce a handle
pub fn consume_sonner() -> SonnerToasts {
    let ctx = consume_context::<SonnerCtx>();
    SonnerToasts {
        add_toast: ctx.add_toast,
        update_toast: ctx.update_toast,
        update_with_options: ctx.update_with_options,
        dismiss_toast: ctx.dismiss_toast,
        delete_toast: ctx.delete_toast,
    }
}

/// Generate unique toast IDs
fn next_toast_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}
