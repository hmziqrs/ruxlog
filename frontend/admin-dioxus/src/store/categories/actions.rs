use super::{
    CategoriesAddPayload, CategoriesEditPayload, CategoriesListQuery, CategoriesState, Category,
};
use crate::services::http_client;
use crate::store::{
    edit_state_abstraction, list_state_abstraction, remove_state_abstraction,
    state_request_abstraction, view_state_abstraction, PaginatedList, StateFrame,
};
use std::collections::HashMap;

impl CategoriesState {
    pub async fn add(&self, payload: CategoriesAddPayload) {
        let meta_payload = payload.clone();
        let request = http_client::post("/category/v1/create", &payload);
        let created = state_request_abstraction(
            &self.add,
            Some(meta_payload),
            request.send(),
            "category",
            |_category: &Category| (None, None),
        )
        .await;

        if created.is_some() {
            self.list().await;
        }
    }

    pub async fn edit(&self, id: i32, payload: CategoriesEditPayload) {
        let _category = edit_state_abstraction(
            &self.edit,
            id,
            payload.clone(),
            http_client::post(&format!("/category/v1/update/{}", id), &payload).send(),
            "category",
            Some(&self.list),
            Some(&self.view),
            |category: &Category| category.id,
            None::<fn(&Category)>,
        )
        .await;
    }

    pub async fn remove(&self, id: i32) {
        let _ = remove_state_abstraction(
            &self.remove,
            id,
            http_client::post(&format!("/category/v1/delete/{}", id), &()).send(),
            "category",
            Some(&self.list),
            Some(&self.view),
            |category: &Category| category.id,
            None::<fn()>,
        )
        .await;
    }

    pub async fn list(&self) {
        let _ = list_state_abstraction::<PaginatedList<Category>>(
            &self.list,
            http_client::post("/category/v1/list/query", &serde_json::json!({})),
            "categories",
        )
        .await;
    }

    pub async fn list_with_query(&self, query: CategoriesListQuery) {
        let _ = list_state_abstraction::<PaginatedList<Category>>(
            &self.list,
            http_client::post("/category/v1/list/query", &query),
            "categories",
        )
        .await;
    }

    pub async fn view(&self, id: i32) {
        let _ = view_state_abstraction(
            &self.view,
            id,
            http_client::get(&format!("/category/v1/view/{}", id)).send(),
            "category",
            |category: &Category| category.clone(),
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
