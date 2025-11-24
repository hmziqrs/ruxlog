use dioxus::prelude::*;
use oxstore::ListQuery;
use ruxlog_shared::store::{SubscriberListQuery, SubscriberStatus};

#[derive(Clone)]
pub struct NewsletterListContext {
    pub selected_ids: Signal<Vec<i32>>,
    pub selected_status: Signal<Option<SubscriberStatus>>,
}

impl NewsletterListContext {
    pub fn new() -> Self {
        Self {
            selected_ids: use_signal(|| Vec::new()),
            selected_status: use_signal(|| None),
        }
    }

    pub fn apply_filters(&mut self, filters: &mut Signal<SubscriberListQuery>) {
        let mut q = filters.peek().clone();
        q.set_page(1);

        q.status = *self.selected_status.peek();

        filters.set(q);
    }

    pub fn clear_all_filters(&mut self, filters: &mut Signal<SubscriberListQuery>) {
        let mut q = filters.peek().clone();
        q.set_page(1);
        q.status = None;
        q.set_search(None);
        filters.set(q);
        self.selected_status.set(None);
    }

    pub fn clear_status_filter(&mut self, filters: &mut Signal<SubscriberListQuery>) {
        self.selected_status.set(None);
        self.apply_filters(filters);
    }

    pub fn active_filter_count(&self, filters: &Signal<SubscriberListQuery>) -> usize {
        let q = filters.read();
        let mut count = 0;
        if q.status.is_some() {
            count += 1;
        }
        count
    }

    pub fn set_status(
        &mut self,
        filters: &mut Signal<SubscriberListQuery>,
        status: Option<SubscriberStatus>,
    ) {
        self.selected_status.set(status);
        self.apply_filters(filters);
    }

    pub fn toggle_subscriber_selection(&mut self, subscriber_id: i32) {
        let mut ids = self.selected_ids.peek().clone();
        if ids.contains(&subscriber_id) {
            ids.retain(|id| *id != subscriber_id);
        } else {
            ids.push(subscriber_id);
        }
        self.selected_ids.set(ids);
    }

    pub fn clear_selections(&mut self) {
        self.selected_ids.set(Vec::new());
    }
}

pub fn use_newsletter_list_context() -> NewsletterListContext {
    use_context::<NewsletterListContext>()
}
