use super::{AclCreatePayload, AclListQuery, AclState, AclSyncResponse, AppConstant};
use oxcore::http;
use oxstore::{
    edit_state_abstraction, list_state_abstraction, remove_state_abstraction,
    state_request_abstraction,
};

impl AclState {
    pub async fn create(&self, payload: AclCreatePayload) {
        let meta = payload.clone();
        let created = state_request_abstraction(
            &self.create,
            Some(meta),
            http::post("/admin/acl/v1/create", &payload).send(),
            "acl",
            |constant: &AppConstant| (Some(constant.clone()), None),
        )
        .await;

        if created.is_some() {
            self.list().await;
        }
    }

    pub async fn update(&self, key: String, payload: AclCreatePayload) {
        let _updated = edit_state_abstraction(
            &self.update,
            key.clone(),
            payload.clone(),
            http::post(&format!("/admin/acl/v1/update/{}", key), &payload).send(),
            "acl",
            Some(&self.list),
            None,
            |constant: &AppConstant| constant.key.clone(),
            None::<fn(&AppConstant)>,
        )
        .await;
    }

    pub async fn remove(&self, key: String) {
        let removed = remove_state_abstraction(
            &self.remove,
            key.clone(),
            http::delete(&format!("/admin/acl/v1/delete/{}", key)).send(),
            "acl",
            Some(&self.list),
            None,
            |constant: &AppConstant| constant.key.clone(),
            None::<fn()>,
        )
        .await;

        if removed {
            self.list().await;
        }
    }

    pub async fn list(&self) {
        let mut default_query = AclListQuery::new();
        default_query.per_page = 20;
        self.list_with_query(default_query).await;
    }

    pub async fn list_with_query(&self, query: AclListQuery) {
        let mut url = format!(
            "/admin/acl/v1/list?page={}&per_page={}",
            query.page, query.per_page
        );
        if let Some(search) = query.search.as_ref() {
            url.push_str(&format!("&search={}", urlencoding::encode(search)));
        }
        if let Some(is_sensitive) = query.is_sensitive {
            url.push_str(&format!("&is_sensitive={}", is_sensitive));
        }
        if let Some(value_type) = query.value_type.as_ref() {
            url.push_str(&format!("&value_type={}", urlencoding::encode(value_type)));
        }

        let _ = list_state_abstraction(&self.list, http::get(&url).send(), "constants").await;
    }

    pub async fn sync(&self) {
        let _ = state_request_abstraction(
            &self.sync,
            None::<()>,
            http::post("/admin/acl/v1/sync", &()).send(),
            "acl_sync",
            |resp: &AclSyncResponse| (Some(resp.clone()), None),
        )
        .await;

        self.list().await;
    }
}
