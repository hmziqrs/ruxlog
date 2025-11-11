use super::{Tag, TagsAddPayload, TagsEditPayload, TagsListQuery, TagsState};
use crate::services::http_client;
use crate::store::{
    edit_state_abstraction, list_state_abstraction, remove_state_abstraction,
    state_request_abstraction, view_state_abstraction, PaginatedList, StateFrame,
};
use std::collections::HashMap;

impl TagsState {
    pub async fn add(&self, payload: TagsAddPayload) {
        let meta_payload = payload.clone();
        let request = http_client::post("/tag/v1/create", &payload);
        let created = state_request_abstraction(
            &self.add,
            Some(meta_payload),
            request.send(),
            "tag",
            |_tag: &Tag| (None, None),
        )
        .await;

        if created.is_some() {
            self.list().await;
        }
    }

    pub async fn edit(&self, id: i32, payload: TagsEditPayload) {
        let _tag = edit_state_abstraction(
            &self.edit,
            id,
            payload.clone(),
            http_client::post(&format!("/tag/v1/update/{}", id), &payload).send(),
            "tag",
            Some(&self.list),
            Some(&self.view),
            |tag: &Tag| tag.id,
            None::<fn(&Tag)>,
        )
        .await;
    }

    pub async fn remove(&self, id: i32) {
        let _ = remove_state_abstraction(
            &self.remove,
            id,
            http_client::post(&format!("/tag/v1/delete/{}", id), &()).send(),
            "tag",
            Some(&self.list),
            Some(&self.view),
            |tag: &Tag| tag.id,
            None::<fn()>,
        )
        .await;
    }

    pub async fn list(&self) {
        let _ = list_state_abstraction::<PaginatedList<Tag>>(
            &self.list,
            http_client::post("/tag/v1/list/query", &serde_json::json!({})),
            "tags",
        )
        .await;
    }

    pub async fn list_with_query(&self, query: TagsListQuery) {
        let _ = list_state_abstraction::<PaginatedList<Tag>>(
            &self.list,
            http_client::post("/tag/v1/list/query", &query),
            "tags",
        )
        .await;
    }

    pub async fn view(&self, id: i32) {
        let _ = view_state_abstraction(
            &self.view,
            id,
            http_client::post(&format!("/tag/v1/view/{}", id), &()).send(),
            "tag",
            |tag: &Tag| tag.clone(),
        )
        .await;
    }

    pub fn reset(&self) {
        *self.add.write() = StateFrame::new();
        *self.edit.write() = HashMap::new();
        *self.remove.write() = HashMap::new();
        *self.list.write() = StateFrame::new();
        *self.view.write() = HashMap::new();
    }
}
