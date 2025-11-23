use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use oxstore::{ListQuery, ListStore, PaginatedList, SortParam, StateFrame};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NewsletterSubscriber {
    pub id: i32,
    pub email: String,
    pub confirmed: bool,
    pub created_at: DateTime<Utc>,
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubscribePayload {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UnsubscribePayload {
    pub email: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfirmPayload {
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SendNewsletterPayload {
    pub subject: String,
    pub content: String,
    pub html_content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SubscriberListQuery {
    pub page: u64,
    pub search: Option<String>,
    pub sorts: Option<Vec<SortParam>>,
    pub created_at_gt: Option<DateTime<Utc>>,
    pub created_at_lt: Option<DateTime<Utc>>,
    pub updated_at_gt: Option<DateTime<Utc>>,
    pub updated_at_lt: Option<DateTime<Utc>>,
}

impl SubscriberListQuery {
    pub fn new() -> Self {
        Self {
            page: 1,
            ..Default::default()
        }
    }
}

impl ListQuery for SubscriberListQuery {
    fn new() -> Self {
        Self::new()
    }
    fn page(&self) -> u64 { self.page }
    fn set_page(&mut self, page: u64) { self.page = page; }
    fn search(&self) -> Option<String> { self.search.clone() }
    fn set_search(&mut self, search: Option<String>) { self.search = search; }
    fn sorts(&self) -> Option<Vec<SortParam>> { self.sorts.clone() }
    fn set_sorts(&mut self, sorts: Option<Vec<SortParam>>) { self.sorts = sorts; }
}

pub struct NewsletterState {
    pub subscribers: GlobalSignal<StateFrame<PaginatedList<NewsletterSubscriber>>>,
    pub subscribe: GlobalSignal<HashMap<String, StateFrame<NewsletterSubscriber, SubscribePayload>>>,
    pub unsubscribe: GlobalSignal<HashMap<String, StateFrame<(), UnsubscribePayload>>>,
    pub confirm: GlobalSignal<HashMap<String, StateFrame<(), ConfirmPayload>>>,
    pub send: GlobalSignal<HashMap<String, StateFrame<(), SendNewsletterPayload>>>,
}

impl ListStore<NewsletterSubscriber, SubscriberListQuery> for NewsletterState {
    fn list_frame(&self) -> &GlobalSignal<StateFrame<PaginatedList<NewsletterSubscriber>>> {
        &self.subscribers
    }
    async fn fetch_list(&self) {
        self.list_subscribers(SubscriberListQuery::new()).await;
    }
    async fn fetch_list_with_query(&self, query: SubscriberListQuery) {
        self.list_subscribers(query).await;
    }
}

impl NewsletterState {
    pub fn new() -> Self {
        Self {
            subscribers: GlobalSignal::new(|| StateFrame::new()),
            subscribe: GlobalSignal::new(HashMap::new),
            unsubscribe: GlobalSignal::new(HashMap::new),
            confirm: GlobalSignal::new(HashMap::new),
            send: GlobalSignal::new(HashMap::new),
        }
    }

    pub fn reset(&self) {
        *self.subscribers.write() = StateFrame::new();
        *self.subscribe.write() = HashMap::new();
        *self.unsubscribe.write() = HashMap::new();
        *self.confirm.write() = HashMap::new();
        *self.send.write() = HashMap::new();
    }
}

static NEWSLETTER_STATE: OnceLock<NewsletterState> = OnceLock::new();

pub fn use_newsletter() -> &'static NewsletterState {
    NEWSLETTER_STATE.get_or_init(NewsletterState::new)
}
