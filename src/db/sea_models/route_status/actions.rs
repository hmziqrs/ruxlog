use super::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, DeleteResult, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use tower_sessions_redis_store::fred::interfaces::{KeysInterface, SetsInterface};

impl Entity {
    pub async fn find_by_pattern(
        db: &DatabaseConnection,
        pattern: &str,
    ) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::RoutePattern.eq(pattern))
            .one(db)
            .await
    }

    pub async fn find_blocked_routes(db: &DatabaseConnection) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .filter(Column::IsBlocked.eq(true))
            .order_by_asc(Column::RoutePattern)
            .all(db)
            .await
    }

    pub async fn create_or_update(
        db: &DatabaseConnection,
        route_pattern: String,
        is_blocked: bool,
        reason: Option<String>,
    ) -> Result<Model, DbErr> {
        if let Some(existing) = Self::find_by_pattern(db, &route_pattern).await? {
            let mut active_model: ActiveModel = existing.into();
            active_model.is_blocked = Set(is_blocked);
            active_model.reason = Set(reason);
            active_model.updated_at = Set(chrono::Utc::now().fixed_offset());
            active_model.update(db).await
        } else {
            let new_route = ActiveModel {
                route_pattern: Set(route_pattern),
                is_blocked: Set(is_blocked),
                reason: Set(reason),
                ..Default::default()
            };
            new_route.insert(db).await
        }
    }

    pub async fn search(
        db: &DatabaseConnection,
        page: u64,
        per_page: u64,
        blocked_only: Option<bool>,
    ) -> Result<(Vec<Model>, u64), DbErr> {
        let mut query = Entity::find();

        if let Some(blocked) = blocked_only {
            query = query.filter(Column::IsBlocked.eq(blocked));
        }

        let total = query.clone().count(db).await?;

        let items = query
            .order_by_asc(Column::RoutePattern)
            .offset((page - 1) * per_page)
            .limit(per_page)
            .all(db)
            .await?;

        Ok((items, total))
    }

    pub async fn delete_by_pattern(
        db: &DatabaseConnection,
        pattern: &str,
    ) -> Result<DeleteResult, DbErr> {
        Entity::delete_many()
            .filter(Column::RoutePattern.eq(pattern))
            .exec(db)
            .await
    }

    pub async fn sync_all_to_redis(
        db: &DatabaseConnection,
        redis_pool: tower_sessions_redis_store::fred::prelude::Pool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let blocked_routes = Self::find_blocked_routes(db).await?;

        let patterns: Vec<String> = blocked_routes
            .iter()
            .map(|r| r.route_pattern.clone())
            .collect();

        redis_pool.del::<(), _>("blocked_routes").await?;

        if !patterns.is_empty() {
            for pattern in patterns {
                redis_pool
                    .sadd::<(), _, _>("blocked_routes", pattern)
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn sync_route_to_redis(
        db: &DatabaseConnection,
        redis_pool: tower_sessions_redis_store::fred::prelude::Pool,
        pattern: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let route = Self::find_by_pattern(db, pattern).await?;

        if let Some(route) = route {
            if route.is_blocked {
                redis_pool
                    .sadd::<(), _, _>("blocked_routes", pattern)
                    .await?;
            } else {
                redis_pool
                    .srem::<(), _, _>("blocked_routes", pattern)
                    .await?;
            }
        } else {
            redis_pool
                .srem::<(), _, _>("blocked_routes", pattern)
                .await?;
        }

        Ok(())
    }
}
