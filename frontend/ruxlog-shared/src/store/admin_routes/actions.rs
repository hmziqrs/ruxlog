use super::{
    AdminRoutesListQuery, AdminRoutesState, BlockRoutePayload, RouteStatus,
    RouteSyncIntervalStatus, UpdateRoutePayload, UpdateSyncIntervalPayload,
};
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
        let payload = serde_json::json!({ "pattern": pattern });
        let _ = state_request_abstraction(
            &self.block,
            None::<BlockRoutePayload>,
            http::post("/admin/route/v1/unblock", &payload).send(),
            "route_status",
            |_route: &RouteStatus| (None, None),
        )
        .await;

        self.list().await;
    }

    pub async fn update(&self, pattern: String, payload: UpdateRoutePayload) {
        let body = serde_json::json!({
            "pattern": pattern,
            "is_blocked": payload.is_blocked,
            "reason": payload.reason
        });
        let updated_route = edit_state_abstraction(
            &self.update,
            pattern.clone(),
            payload.clone(),
            http::post("/admin/route/v1/update", &body).send(),
            "route_status",
            None,
            None,
            |route: &RouteStatus| route.route_pattern.clone(),
            None::<fn(&RouteStatus)>,
        )
        .await;

        if updated_route.is_some() {
            self.list().await;
        }
    }

    pub async fn remove(&self, pattern: String) {
        let payload = serde_json::json!({ "pattern": pattern });
        let removed = remove_state_abstraction(
            &self.remove,
            pattern.clone(),
            http::post("/admin/route/v1/delete", &payload).send(),
            "route_status",
            None,
            None,
            |route: &RouteStatus| route.route_pattern.clone(),
            None::<fn()>,
        )
        .await;

        if removed {
            self.list().await;
        }
    }

    pub async fn list(&self) {
        let query = AdminRoutesListQuery::new();
        self.list_with_query(query).await;
    }

    pub async fn list_with_query(&self, query: AdminRoutesListQuery) {
        let _ = list_state_abstraction(
            &self.list,
            http::post("/admin/route/v1/list", &query).send(),
            "data",
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
        self.fetch_sync_interval().await;
    }

    pub async fn fetch_sync_interval(&self) {
        let _ = state_request_abstraction(
            &self.sync_interval,
            None::<UpdateSyncIntervalPayload>,
            http::get("/admin/route/v1/sync_interval").send(),
            "interval_secs",
            |status: &RouteSyncIntervalStatus| (Some(status.clone()), None),
        )
        .await;
    }

    pub async fn update_sync_interval(&self, payload: UpdateSyncIntervalPayload) {
        let _ = state_request_abstraction(
            &self.sync_interval,
            Some(payload.clone()),
            http::post("/admin/route/v1/sync_interval", &payload).send(),
            "interval_secs",
            |status: &RouteSyncIntervalStatus| (Some(status.clone()), None),
        )
        .await;
    }

    pub async fn pause_sync_interval(&self) {
        let _ = state_request_abstraction(
            &self.sync_interval,
            None::<UpdateSyncIntervalPayload>,
            http::post("/admin/route/v1/sync_interval/pause", &()).send(),
            "interval_secs",
            |status: &RouteSyncIntervalStatus| (Some(status.clone()), None),
        )
        .await;
    }

    pub async fn resume_sync_interval(&self) {
        let _ = state_request_abstraction(
            &self.sync_interval,
            None::<UpdateSyncIntervalPayload>,
            http::post("/admin/route/v1/sync_interval/resume", &()).send(),
            "interval_secs",
            |status: &RouteSyncIntervalStatus| (Some(status.clone()), None),
        )
        .await;
    }

    pub async fn restart_sync_interval(&self) {
        let _ = state_request_abstraction(
            &self.sync_interval,
            None::<UpdateSyncIntervalPayload>,
            http::post("/admin/route/v1/sync_interval/restart", &()).send(),
            "interval_secs",
            |status: &RouteSyncIntervalStatus| (Some(status.clone()), None),
        )
        .await;
    }
}
