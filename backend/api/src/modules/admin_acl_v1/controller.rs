use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use serde_json::json;
use tracing::{error, info};

use crate::{
    error::ErrorResponse, extractors::ValidatedJson, services::acl_service::AclService,
    services::acl_service::ConstantsListParams, services::acl_service::UpsertConstantPayload,
    services::auth::AuthSession, AppState,
};

use super::validator::{ConstantsListQuery, UpsertConstantRequest};

#[debug_handler]
pub async fn import_env_constants(
    State(state): State<AppState>,
    _auth: AuthSession,
) -> Result<impl IntoResponse, ErrorResponse> {
    let result = AclService::bootstrap_from_env(State(state)).await;

    match result {
        Ok(payload) => {
            info!("Imported env constants into ACL");
            Ok((StatusCode::OK, Json(payload)))
        }
        Err(err) => {
            error!(error = %err, "Failed to import env constants");
            Err(err)
        }
    }
}

#[debug_handler]
pub async fn sync_constants(
    State(state): State<AppState>,
    _auth: AuthSession,
) -> Result<impl IntoResponse, ErrorResponse> {
    let result = AclService::sync_all_to_redis(State(state)).await;
    match result {
        Ok(payload) => {
            info!("Synced ACL constants to Redis");
            Ok((StatusCode::OK, Json(payload)))
        }
        Err(err) => {
            error!(error = %err, "Failed to sync ACL constants to Redis");
            Err(err)
        }
    }
}

#[debug_handler]
pub async fn list_constants(
    State(state): State<AppState>,
    _auth: AuthSession,
    Query(query): Query<ConstantsListQuery>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let params = ConstantsListParams {
        page: query.page,
        per_page: query.per_page,
        search: query.search,
        is_sensitive: query.is_sensitive,
        value_type: query.value_type,
    };

    let result = AclService::list_constants(State(state), params).await;

    match result {
        Ok((items, total)) => {
            info!(count = items.len(), "Listed ACL constants");
            let sanitized: Vec<serde_json::Value> = items
                .into_iter()
                .map(|item| {
                    json!({
                        "key": item.key,
                        "value": if item.is_sensitive { serde_json::Value::String("********".into()) } else { serde_json::Value::String(item.value) },
                        "value_type": item.value_type,
                        "description": item.description,
                        "is_sensitive": item.is_sensitive,
                        "source": item.source,
                        "updated_at": item.updated_at,
                        "created_at": item.created_at,
                        "updated_by": item.updated_by,
                    })
                })
                .collect();

            Ok(Json(json!({
                "data": sanitized,
                "total": total,
                "page": query.page.unwrap_or(1),
                "per_page": query.per_page.unwrap_or(20)
            })))
        }
        Err(err) => Err(err),
    }
}

#[debug_handler]
pub async fn get_constant(
    State(state): State<AppState>,
    _auth: AuthSession,
    Path(key): Path<String>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let result = AclService::get_constant(State(state), &key).await;
    match result {
        Ok(item) => Ok(Json(json!({
            "key": item.key,
            "value": if item.is_sensitive { "********".to_string() } else { item.value },
            "value_type": item.value_type,
            "description": item.description,
            "is_sensitive": item.is_sensitive,
            "source": item.source,
            "updated_at": item.updated_at,
            "created_at": item.created_at,
            "updated_by": item.updated_by,
        }))),
        Err(err) => Err(err),
    }
}

#[debug_handler]
pub async fn create_constant(
    State(state): State<AppState>,
    auth: AuthSession,
    payload: ValidatedJson<UpsertConstantRequest>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let req = payload.0;
    let body = UpsertConstantPayload {
        key: req.key,
        value: req.value,
        value_type: req.value_type,
        description: req.description,
        is_sensitive: req.is_sensitive.unwrap_or(false),
        source: Some("manual".to_string()),
    };

    let result = AclService::upsert_constant(State(state), body, auth.user.map(|u| u.id)).await;

    match result {
        Ok(item) => Ok((StatusCode::CREATED, Json(json!({ "data": sanitize(item) })))),
        Err(err) => Err(err),
    }
}

#[debug_handler]
pub async fn update_constant(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(key): Path<String>,
    payload: ValidatedJson<UpsertConstantRequest>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let req = payload.0;
    let body = UpsertConstantPayload {
        key: key.clone(),
        value: req.value,
        value_type: req.value_type,
        description: req.description,
        is_sensitive: req.is_sensitive.unwrap_or(false),
        source: Some("manual".to_string()),
    };

    let result = AclService::upsert_constant(State(state), body, auth.user.map(|u| u.id)).await;

    match result {
        Ok(item) => Ok(Json(json!({ "data": sanitize(item) }))),
        Err(err) => Err(err),
    }
}

#[debug_handler]
pub async fn delete_constant(
    State(state): State<AppState>,
    _auth: AuthSession,
    Path(key): Path<String>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let result = AclService::delete_constant(State(state), key.clone()).await;
    match result {
        Ok(_) => Ok(Json(json!({ "message": "Deleted", "key": key }))),
        Err(err) => Err(err),
    }
}

fn sanitize(item: crate::db::sea_models::app_constant::Model) -> serde_json::Value {
    json!({
        "key": item.key,
        "value": if item.is_sensitive { "********".to_string() } else { item.value },
        "value_type": item.value_type,
        "description": item.description,
        "is_sensitive": item.is_sensitive,
        "source": item.source,
        "updated_at": item.updated_at,
        "created_at": item.created_at,
        "updated_by": item.updated_by,
    })
}
