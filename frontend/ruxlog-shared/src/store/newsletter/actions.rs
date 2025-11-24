use super::{
    ConfirmPayload, NewsletterState, SendNewsletterPayload, SubscribePayload, SubscriberListQuery,
    UnsubscribePayload,
};
use oxcore::http;
use oxstore::list_state_abstraction;

impl NewsletterState {
    pub async fn subscribe(&self, payload: SubscribePayload) {
        let email = payload.email.clone();

        match http::post("/newsletter/v1/subscribe", &payload)
            .send()
            .await
        {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    let mut map = self.subscribe.write();
                    if let Ok(subscriber) = response.json::<super::NewsletterSubscriber>().await {
                        let mut frame = oxstore::StateFrame::new();
                        frame.data = Some(subscriber);
                        frame.meta = Some(payload);
                        map.insert(email, frame);
                    }
                    drop(map);
                    self.list_subscribers(SubscriberListQuery::new()).await;
                }
            }
            Err(_) => {}
        }
    }

    pub async fn unsubscribe(&self, payload: UnsubscribePayload) {
        let email = payload.email.clone().unwrap_or_default();

        match http::post("/newsletter/v1/unsubscribe", &payload)
            .send()
            .await
        {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    let mut map = self.unsubscribe.write();
                    let mut frame = oxstore::StateFrame::new();
                    frame.data = Some(());
                    frame.meta = Some(payload);
                    map.insert(email, frame);
                    drop(map);
                    self.list_subscribers(SubscriberListQuery::new()).await;
                }
            }
            Err(_) => {}
        }
    }

    pub async fn confirm(&self, payload: ConfirmPayload) {
        let token = payload.token.clone();

        match http::post("/newsletter/v1/confirm", &payload).send().await {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    let mut map = self.confirm.write();
                    let mut frame = oxstore::StateFrame::new();
                    frame.data = Some(());
                    frame.meta = Some(payload);
                    map.insert(token, frame);
                    drop(map);
                    self.list_subscribers(SubscriberListQuery::new()).await;
                }
            }
            Err(_) => {}
        }
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

        match http::post("/newsletter/v1/send", &payload).send().await {
            Ok(response) => {
                if (200..300).contains(&response.status()) {
                    let mut map = self.send.write();
                    let mut frame = oxstore::StateFrame::new();
                    frame.data = Some(());
                    frame.meta = Some(payload);
                    map.insert(subject, frame);
                }
            }
            Err(_) => {}
        }
    }
}
