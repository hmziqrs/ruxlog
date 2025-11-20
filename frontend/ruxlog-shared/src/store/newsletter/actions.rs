use super::{
    ConfirmPayload, NewsletterState, SendNewsletterPayload, SendResult, SubscribePayload,
    SubscriberListQuery, UnsubscribePayload,
};
use oxcore::http;
use oxstore::{list_state_abstraction, state_request_abstraction};

impl NewsletterState {
    pub async fn subscribe(&self, payload: SubscribePayload) {
        let meta = payload.clone();
        let created = state_request_abstraction(
            &self.subscribe,
            Some(meta),
            http::post("/newsletter/v1/subscribe", &payload).send(),
            "subscriber",
            |subscriber: &super::NewsletterSubscriber| (Some(subscriber.clone()), None),
        )
        .await;

        if created.is_some() {
            self.list_subscribers(SubscriberListQuery::new()).await;
        }
    }

    pub async fn unsubscribe(&self, payload: UnsubscribePayload) {
        let meta = payload.clone();
        let _ = state_request_abstraction(
            &self.unsubscribe,
            Some(meta),
            http::post("/newsletter/v1/unsubscribe", &payload).send(),
            "unsubscribe",
            |_res: &serde_json::Value| (Some(Some(())), None),
        )
        .await;

        self.list_subscribers(SubscriberListQuery::new()).await;
    }

    pub async fn confirm(&self, payload: ConfirmPayload) {
        let meta = payload.clone();
        let _ = state_request_abstraction(
            &self.confirm,
            Some(meta),
            http::post("/newsletter/v1/confirm", &payload).send(),
            "confirm",
            |_res: &serde_json::Value| (Some(Some(())), None),
        )
        .await;
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
        let meta = payload.clone();
        let _ = state_request_abstraction(
            &self.send_status,
            Some(meta),
            http::post("/newsletter/v1/send", &payload).send(),
            "send_newsletter",
            |res: &SendResult| (Some(Some(res.clone())), None),
        )
        .await;
    }
}
