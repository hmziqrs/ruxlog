use super::{
    ConfirmPayload, NewsletterState, SendNewsletterPayload, SubscribePayload,
    SubscriberListQuery, UnsubscribePayload,
};
use oxcore::http;
use oxstore::{edit_state_abstraction, list_state_abstraction, state_request_abstraction};
use std::collections::HashMap;

impl NewsletterState {
    pub async fn subscribe(&self, payload: SubscribePayload) {
        let email = payload.email.clone();
        let _subscriber = edit_state_abstraction(
            &self.subscribe,
            email.clone(),
            payload.clone(),
            http::post("/newsletter/v1/subscribe", &payload).send(),
            "subscriber",
            Some(&self.subscribers),
            None::<&HashMap<String, oxstore::StateFrame<super::NewsletterSubscriber>>>,
            |subscriber: &super::NewsletterSubscriber| subscriber.email.clone(),
            None::<fn(&super::NewsletterSubscriber)>,
        )
        .await;

        self.list_subscribers(SubscriberListQuery::new()).await;
    }

    pub async fn unsubscribe(&self, payload: UnsubscribePayload) {
        let email = payload.email.clone().unwrap_or_default();
        let _ = edit_state_abstraction(
            &self.unsubscribe,
            email.clone(),
            payload.clone(),
            http::post("/newsletter/v1/unsubscribe", &payload).send(),
            "unsubscribe",
            Some(&self.subscribers),
            None::<&HashMap<String, oxstore::StateFrame<()>>>,
            |_res: &serde_json::Value| String::new(),
            None::<fn(&serde_json::Value)>,
        )
        .await;

        self.list_subscribers(SubscriberListQuery::new()).await;
    }

    pub async fn confirm(&self, payload: ConfirmPayload) {
        let token = payload.token.clone();
        let _ = edit_state_abstraction(
            &self.confirm,
            token.clone(),
            payload.clone(),
            http::post("/newsletter/v1/confirm", &payload).send(),
            "confirm",
            Some(&self.subscribers),
            None::<&HashMap<String, oxstore::StateFrame<()>>>,
            |_res: &serde_json::Value| String::new(),
            None::<fn(&serde_json::Value)>,
        )
        .await;

        self.list_subscribers(SubscriberListQuery::new()).await;
    }

    pub async fn list_subscribers(&self, query: SubscriberListQuery) {
        let _ = list_state_abstraction(
            &self.subscribers,
            http::post("/newsletter/v1/subscribers/list", &query).send(),
            "subscribers",
        )
        .await;
    }

    pub async fn send(&self, payload: SendNewsletterPayload) {
        let subject = payload.subject.clone();
        let _ = edit_state_abstraction(
            &self.send,
            subject.clone(),
            payload.clone(),
            http::post("/newsletter/v1/send", &payload).send(),
            "send_newsletter",
            None::<&oxstore::StateFrame<oxstore::PaginatedList<super::NewsletterSubscriber>>>,
            None::<&HashMap<String, oxstore::StateFrame<()>>>,
            |_res: &serde_json::Value| String::new(),
            None::<fn(&serde_json::Value)>,
        )
        .await;
    }
}
