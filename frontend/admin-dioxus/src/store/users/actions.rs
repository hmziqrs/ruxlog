use super::{User, UsersAddPayload, UsersEditPayload, UsersListQuery, UsersState};
use crate::services::http_client;
use crate::store::{
    edit_state_abstraction, list_state_abstraction, remove_state_abstraction,
    state_request_abstraction, view_state_abstraction, PaginatedList, StateFrame,
};
use std::collections::HashMap;

impl UsersState {
    pub async fn add(&self, payload: UsersAddPayload) {
        let meta_payload = payload.clone();
        let request = http_client::post("/user/v1/admin/create", &payload);
        let created = state_request_abstraction(
            &self.add,
            Some(meta_payload),
            request.send(),
            "user",
            |_user: &User| (None, None),
        )
        .await;

        if created.is_some() {
            self.list().await;
        }
    }

    pub async fn edit(&self, id: i32, payload: UsersEditPayload) {
        let _user = edit_state_abstraction(
            &self.edit,
            id,
            payload.clone(),
            http_client::post(&format!("/user/v1/admin/update/{}", id), &payload).send(),
            "user",
            Some(&self.list),
            Some(&self.view),
            |user: &User| user.id,
            None::<fn(&User)>,
        )
        .await;
    }

    pub async fn remove(&self, id: i32) {
        let _ = remove_state_abstraction(
            &self.remove,
            id,
            http_client::post(&format!("/user/v1/admin/delete/{}", id), &()).send(),
            "user",
            Some(&self.list),
            Some(&self.view),
            |user: &User| user.id,
            None::<fn()>,
        )
        .await;
    }

    pub async fn list(&self) {
        let _ = list_state_abstraction::<PaginatedList<User>>(
            &self.list,
            http_client::post("/user/v1/admin/list", &serde_json::json!({})),
            "users",
        )
        .await;
    }

    pub async fn list_with_query(&self, query: UsersListQuery) {
        let _ = list_state_abstraction::<PaginatedList<User>>(
            &self.list,
            http_client::post("/user/v1/admin/list", &query),
            "users",
        )
        .await;
    }

    pub async fn view(&self, id: i32) {
        let _ = view_state_abstraction(
            &self.view,
            id,
            http_client::post(&format!("/user/v1/admin/view/{}", id), &()).send(),
            "user",
            |user: &User| user.clone(),
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
