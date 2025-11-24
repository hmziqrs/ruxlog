use crate::db::sea_models::app_constant::Entity as AppConstant;
use crate::db::sea_models::app_constant::Model as AppConstantModel;
use crate::error::{ErrorCode, ErrorResponse};
use crate::state::AppState;
use axum::extract::State;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;
use tower_sessions_redis_store::fred::prelude::*;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertConstantPayload {
    pub key: String,
    pub value: String,
    pub value_type: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub is_sensitive: bool,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstantsListParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub search: Option<String>,
    pub is_sensitive: Option<bool>,
    pub value_type: Option<String>,
}

pub struct AclService;

impl AclService {
    pub const VALUE_HASH: &'static str = "app_constants";
    pub const META_HASH: &'static str = "app_constants_meta";

    pub async fn bootstrap_from_env(
        State(state): State<AppState>,
    ) -> Result<serde_json::Value, ErrorResponse> {
        let env_vars: Vec<(String, String)> = std::env::vars().collect();
        for (key, value) in env_vars {
            let normalized_key = key.trim().to_string();
            if normalized_key.is_empty() {
                continue;
            }
            let is_sensitive =
                Self::guess_sensitive(&normalized_key) || Self::guess_sensitive(&value);

            let _ = AppConstant::ensure_exists(
                &state.sea_db,
                &normalized_key,
                &value,
                None,
                is_sensitive,
                "env",
            )
            .await
            .map_err(|e| {
                ErrorResponse::new(ErrorCode::InternalServerError).with_message(e.to_string())
            })?;
        }

        Self::sync_all_to_redis(State(state)).await?;

        Ok(json!({"message": "Env constants bootstrapped"}))
    }

    fn guess_sensitive(s: &str) -> bool {
        let lower = s.to_ascii_lowercase();
        lower.contains("secret")
            || lower.contains("password")
            || lower.contains("token")
            || lower.contains("key")
            || lower.contains("access")
    }

    pub async fn get_constant(
        State(state): State<AppState>,
        key: &str,
    ) -> Result<AppConstantModel, ErrorResponse> {
        let redis_value: Option<String> = state
            .redis_pool
            .hget(Self::VALUE_HASH, key)
            .await
            .unwrap_or(None);

        if let Some(value) = redis_value {
            if let Some(meta_json) = state
                .redis_pool
                .hget::<Option<String>, _, _>(Self::META_HASH, key)
                .await
                .unwrap_or(None)
            {
                if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&meta_json) {
                    let is_sensitive = meta
                        .get("is_sensitive")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let model = AppConstantModel {
                        id: 0,
                        key: key.to_string(),
                        value,
                        value_type: meta
                            .get("value_type")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                        description: None,
                        is_sensitive,
                        source: "cache".to_string(),
                        updated_by: None,
                        created_at: chrono::Utc::now().fixed_offset(),
                        updated_at: chrono::Utc::now().fixed_offset(),
                    };
                    return Ok(model);
                }
            }
        }

        let from_db = AppConstant::find_by_key(&state.sea_db, key)
            .await
            .map_err(|e| {
                ErrorResponse::new(ErrorCode::InternalServerError).with_message(e.to_string())
            })?
            .ok_or_else(|| {
                ErrorResponse::new(ErrorCode::RecordNotFound).with_message("Key not found")
            })?;

        Self::write_single_to_redis(&state, &from_db)
            .await
            .map_err(|e| {
                ErrorResponse::new(ErrorCode::InternalServerError).with_message(e.to_string())
            })?;

        Ok(from_db)
    }

    pub async fn list_constants(
        State(state): State<AppState>,
        params: ConstantsListParams,
    ) -> Result<(Vec<AppConstantModel>, u64), ErrorResponse> {
        let page = params.page.unwrap_or(1);
        let per_page = params.per_page.unwrap_or(20);
        AppConstant::list(
            &state.sea_db,
            page,
            per_page,
            params.search,
            params.is_sensitive,
            params.value_type,
        )
        .await
        .map_err(|e| ErrorResponse::new(ErrorCode::InternalServerError).with_message(e.to_string()))
    }

    pub async fn upsert_constant(
        State(state): State<AppState>,
        payload: UpsertConstantPayload,
        updated_by: Option<i32>,
    ) -> Result<AppConstantModel, ErrorResponse> {
        let key = payload.key.trim().to_string();
        if key.is_empty() {
            return Err(
                ErrorResponse::new(ErrorCode::InvalidInput).with_message("Key cannot be empty")
            );
        }

        let value = payload.value;
        let value_type = payload.value_type.clone();
        let description = payload.description.clone();
        let is_sensitive = payload.is_sensitive;
        let source = payload.source.unwrap_or_else(|| "manual".to_string());

        let model = AppConstant::upsert_value(
            &state.sea_db,
            &key,
            &value,
            value_type.clone(),
            description,
            is_sensitive,
            &source,
            updated_by,
        )
        .await
        .map_err(|e| {
            ErrorResponse::new(ErrorCode::InternalServerError).with_message(e.to_string())
        })?;

        Self::write_single_to_redis(&state, &model)
            .await
            .map_err(|e| {
                ErrorResponse::new(ErrorCode::InternalServerError).with_message(e.to_string())
            })?;

        info!(key, "ACL constant upserted and cached");
        Ok(model)
    }

    pub async fn delete_constant(
        State(state): State<AppState>,
        key: String,
    ) -> Result<(), ErrorResponse> {
        AppConstant::delete_by_key(&state.sea_db, &key)
            .await
            .map_err(|e| {
                ErrorResponse::new(ErrorCode::InternalServerError).with_message(e.to_string())
            })?;

        state
            .redis_pool
            .hdel::<(), _, _>(Self::VALUE_HASH, &key)
            .await
            .ok();
        state
            .redis_pool
            .hdel::<(), _, _>(Self::META_HASH, &key)
            .await
            .ok();

        Ok(())
    }

    pub async fn sync_all_to_redis(
        State(state): State<AppState>,
    ) -> Result<serde_json::Value, ErrorResponse> {
        AppConstant::sync_all_to_redis(
            &state.sea_db,
            state.redis_pool,
            Self::VALUE_HASH,
            Self::META_HASH,
        )
        .await
        .map_err(|e| {
            ErrorResponse::new(ErrorCode::InternalServerError).with_message(e.to_string())
        })?;

        Ok(json!({"message": "ACL cache synced to redis"}))
    }

    async fn write_single_to_redis(
        state: &AppState,
        model: &AppConstantModel,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        state
            .redis_pool
            .hset::<(), _, _>(Self::VALUE_HASH, vec![(&model.key, &model.value)])
            .await?;

        let meta = serde_json::json!({
            "value_type": model.value_type,
            "is_sensitive": model.is_sensitive,
            "updated_at": model.updated_at,
        })
        .to_string();

        state
            .redis_pool
            .hset::<(), _, _>(Self::META_HASH, vec![(&model.key, meta)])
            .await?;

        Ok(())
    }
}
