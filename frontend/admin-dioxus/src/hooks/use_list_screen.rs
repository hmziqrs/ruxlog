use crate::store::ListQuery;
use crate::types::{Order, SortParam};
use dioxus::prelude::*;
use gloo_timers::future::sleep;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ListScreenConfig {
    pub default_sort_field: String,
    pub default_sort_order: Order,
}

impl Default for ListScreenConfig {
    fn default() -> Self {
        Self {
            default_sort_field: "name".to_string(),
            default_sort_order: Order::Asc,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ListScreenState {
    pub search_input: Signal<String>,
    pub sort_field: Signal<String>,
    pub sort_order: Signal<Order>,
    pub reload_tick: Signal<u32>,
}

impl ListScreenState {
    pub fn new(config: &ListScreenConfig) -> Self {
        Self {
            search_input: use_signal(|| String::new()),
            sort_field: use_signal(|| config.default_sort_field.clone()),
            sort_order: use_signal(|| config.default_sort_order.clone()),
            reload_tick: use_signal(|| 0u32),
        }
    }

    // Direct signal access methods
    pub fn search_input(&self) -> String {
        self.search_input.read().clone()
    }

    pub fn sort_field(&self) -> String {
        self.sort_field.read().clone()
    }

    pub fn sort_order(&self) -> Order {
        self.sort_order.read().clone()
    }

    pub fn reload_tick(&self) -> u32 {
        self.reload_tick.read().clone()
    }

    // Utility methods for common operations
    pub fn get_sort_params(&self) -> Vec<SortParam> {
        vec![SortParam {
            field: self.sort_field.read().clone(),
            order: self.sort_order.read().clone(),
        }]
    }

    pub fn handle_sort(&self, field: String) {
        let mut sort_field = self.sort_field;
        let mut sort_order = self.sort_order;

        let current_field = sort_field();
        let current_order = sort_order();

        if current_field == field {
            // Toggle order for same field
            let new_order = match current_order {
                Order::Asc => Order::Desc,
                Order::Desc => Order::Asc,
            };
            sort_order.set(new_order);
        } else {
            // New field, default to asc
            sort_field.set(field);
            sort_order.set(Order::Asc);
        }
    }

    pub fn set_search(&self, value: String) {
        let mut search_input = self.search_input;
        search_input.set(value);
    }

    pub fn clear_search(&self) {
        let mut search_input = self.search_input;
        search_input.set(String::new());
    }

    pub fn trigger_reload(&self) {
        let mut reload_tick = self.reload_tick;
        let current = reload_tick();
        reload_tick.set(current + 1);
    }
}

/// Generic hook for list screen logic
///
/// Usage:
/// ```rust
/// let list_state = use_list_screen(Some(ListScreenConfig::default()));
/// ```
pub fn use_list_screen(config: Option<ListScreenConfig>) -> ListScreenState {
    let config = config.unwrap_or_default();
    ListScreenState::new(&config)
}

/// Enhanced list screen handlers for stores implementing ListStore trait
pub struct ListScreenHandlers {
    pub handle_sort: EventHandler<String>,
    pub handle_search: EventHandler<String>,
    pub handle_retry: EventHandler<()>,
    pub handle_prev: EventHandler<u64>,
    pub handle_next: EventHandler<u64>,
    pub handle_clear: EventHandler<()>,
}

/// Enhanced hook that creates handlers for list operations
/// This eliminates boilerplate in list screen components
pub fn use_list_screen_with_handlers<Q>(
    config: Option<ListScreenConfig>,
    filters: Signal<Q>,
) -> (ListScreenState, ListScreenHandlers)
where
    Q: ListQuery + 'static,
{
    let list_state = use_list_screen(config);
    // Debounce generation counter to ensure only the latest input applies
    let debounce_gen = use_signal(|| 0u64);

    // Create handlers
    let handle_sort = {
        let list_state = list_state;
        let mut filters = filters;
        move |field: String| {
            list_state.handle_sort(field);
            let mut q = filters.peek().clone();
            q.set_page(1);
            q.set_sorts(Some(list_state.get_sort_params()));
            filters.set(q);
        }
    };

    let handle_search = {
        let mut filters = filters;
        let list_state_local = list_state;
        let mut debounce_gen = debounce_gen;
        move |val: String| {
            // Keep the controlled input in sync immediately
            list_state_local.set_search(val.clone());

            // Advance debounce generation and capture this tick
            let this_tick = debounce_gen() + 1;
            debounce_gen.set(this_tick);

            // Only the latest tick should win after the delay
            spawn(async move {
                sleep(Duration::from_millis(500)).await;
                // If another keystroke happened since we scheduled this, abort
                if debounce_gen() != this_tick {
                    return;
                }
                let mut q = filters.peek().clone();
                q.set_page(1);
                q.set_search(if val.is_empty() { None } else { Some(val) });
                filters.set(q);
            });
        }
    };

    let handle_retry = {
        let list_state = list_state;
        move |_| {
            list_state.trigger_reload();
        }
    };

    let handle_prev = {
        let mut filters = filters;
        move |current_page: u64| {
            let new_page = current_page.saturating_sub(1).max(1);
            let mut q = filters.peek().clone();
            q.set_page(new_page);
            filters.set(q);
        }
    };

    let handle_next = {
        let mut filters = filters;
        move |current_page: u64| {
            let new_page = current_page + 1;
            let mut q = filters.peek().clone();
            q.set_page(new_page);
            filters.set(q);
        }
    };

    let handle_clear = {
        let list_state = list_state;
        let mut filters = filters;
        move |_| {
            list_state.clear_search();
            filters.set(Q::new());
        }
    };

    let handlers = ListScreenHandlers {
        handle_sort: EventHandler::new(handle_sort),
        handle_search: EventHandler::new(handle_search),
        handle_retry: EventHandler::new(handle_retry),
        handle_prev: EventHandler::new(handle_prev),
        handle_next: EventHandler::new(handle_next),
        handle_clear: EventHandler::new(handle_clear),
    };

    (list_state, handlers)
}
