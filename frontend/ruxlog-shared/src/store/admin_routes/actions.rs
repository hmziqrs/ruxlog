use super::{AdminRoutesState, BlockRoutePayload, RouteStatus, UpdateRoutePayload};
use oxcore::http;
use oxstore::{
    edit_state_abstraction, list_state_abstraction, remove_state_abstraction,
    state_request_abstraction,
};

impl AdminRoutesState {
    pub async fn block(&self, payload: BlockRoutePayload) {
        let meta = payload.clone();
        let created = state_request_abstraction(
            &self.block,
            Some(meta),
            http::post("/admin/route/v1/block", &payload).send(),
            "route_status",
            |route: &RouteStatus| (Some(route.clone()), None),
        )
        .await;

        if created.is_some() {
            self.list().await;
        }
    }

    pub async fn unblock(&self, pattern: String) {
        let _ = state_request_abstraction(
            &self.block,
            None::<BlockRoutePayload>,
            http::post(&format!("/admin/route/v1/unblock/{}", pattern), &()).send(),
            "route_status",
            |_route: &RouteStatus| (None, None),
        )
        .await;

        self.list().await;
    }

    pub async fn update(&self, pattern: String, payload: UpdateRoutePayload) {
        let updated_route = edit_state_abstraction(
            &self.update,
            pattern.clone(),
            payload.clone(),
            http::post(&format!("/admin/route/v1/update/{}", pattern), &payload).send(),
            "route_status",
            None,
            None,
            |route: &RouteStatus| route.pattern.clone(),
            None::<fn(&RouteStatus)>,
        )
        .await;

        if updated_route.is_some() {
            self.list().await;
        }
    }

    pub async fn remove(&self, pattern: String) {
        let removed = remove_state_abstraction(
            &self.remove,
            pattern.clone(),
            http::delete(&format!("/admin/route/v1/delete/{}", pattern)).send(),
            "route_status",
            None,
            None,
            |route: &RouteStatus| route.pattern.clone(),
            None::<fn()>,
        )
        .await;

        if removed {
            self.list().await;
        }
    }

    pub async fn list(&self) {
        let _ = list_state_abstraction(
            &self.list,
            http::get("/admin/route/v1/list").send(),
            "routes",
        )
        .await;
    }

    pub async fn sync(&self) {
        let _ = state_request_abstraction(
            &self.sync,
            None::<()>,
            http::get("/admin/route/v1/sync").send(),
            "route_sync",
            |_resp: &serde_json::Value| (Some(()), None),
        )
        .await;

        self.list().await;
    }
}
