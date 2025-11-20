use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use oxstore::{PaginatedList, StateFrame};
use serde::{Deserialize, Serialize};
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
    pub confirmed: Option<bool>,
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct SendResult {
    pub success: bool,
    pub message: Option<String>,
}

pub struct NewsletterState {
    pub subscribers: GlobalSignal<StateFrame<PaginatedList<NewsletterSubscriber>>>,
    pub subscribe: GlobalSignal<StateFrame<NewsletterSubscriber, SubscribePayload>>,
    pub unsubscribe: GlobalSignal<StateFrame<Option<()>, UnsubscribePayload>>,
    pub confirm: GlobalSignal<StateFrame<Option<()>, ConfirmPayload>>,
    pub send_status: GlobalSignal<StateFrame<Option<SendResult>, SendNewsletterPayload>>,
}

impl NewsletterState {
    pub fn new() -> Self {
        Self {
            subscribers: GlobalSignal::new(|| StateFrame::new()),
            subscribe: GlobalSignal::new(|| StateFrame::new()),
            unsubscribe: GlobalSignal::new(|| StateFrame::new()),
            confirm: GlobalSignal::new(|| StateFrame::new()),
            send_status: GlobalSignal::new(|| StateFrame::new()),
        }
    }

    pub fn reset(&self) {
        *self.subscribers.write() = StateFrame::new();
        *self.subscribe.write() = StateFrame::new();
        *self.unsubscribe.write() = StateFrame::new();
        *self.confirm.write() = StateFrame::new();
        *self.send_status.write() = StateFrame::new();
    }
}

static NEWSLETTER_STATE: OnceLock<NewsletterState> = OnceLock::new();

pub fn use_newsletter() -> &'static NewsletterState {
    NEWSLETTER_STATE.get_or_init(NewsletterState::new)
}
