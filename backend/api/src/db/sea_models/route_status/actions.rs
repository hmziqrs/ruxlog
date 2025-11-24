use super::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, DeleteResult, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use tower_sessions_redis_store::fred::interfaces::{KeysInterface, SetsInterface};

impl Entity {
    pub const PER_PAGE: u64 = 20;
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

    pub async fn ensure_exists(
        db: &DatabaseConnection,
        route_pattern: &str,
    ) -> Result<Model, DbErr> {
        if let Some(existing) = Self::find_by_pattern(db, route_pattern).await? {
            return Ok(existing);
        }

        let new_route = ActiveModel {
            route_pattern: Set(route_pattern.to_string()),
            is_blocked: Set(false),
            reason: Set(None),
            ..Default::default()
        };

        match new_route.insert(db).await {
            Ok(model) => Ok(model),
            Err(DbErr::Exec(exec_err)) => match Self::find_by_pattern(db, route_pattern).await? {
                Some(existing) => Ok(existing),
                None => Err(DbErr::Exec(exec_err)),
            },
            Err(err) => Err(err),
        }
    }

    pub async fn search(
        db: &DatabaseConnection,
        query: RouteStatusQuery,
    ) -> Result<(Vec<Model>, u64), DbErr> {
        let mut route_query = Entity::find();

        match BlockFilter::resolve(query.block_filter) {
            BlockFilter::All => {}
            BlockFilter::Blocked => {
                route_query = route_query.filter(Column::IsBlocked.eq(true));
            }
            BlockFilter::Unblocked => {
                route_query = route_query.filter(Column::IsBlocked.eq(false));
            }
        }

        if let Some(search_term) = &query.search {
            route_query = route_query.filter(Column::RoutePattern.contains(search_term));
        }

        if let Some(ts) = query.created_at_gt {
            route_query = route_query.filter(Column::CreatedAt.gt(ts));
        }
        if let Some(ts) = query.created_at_lt {
            route_query = route_query.filter(Column::CreatedAt.lt(ts));
        }
        if let Some(ts) = query.updated_at_gt {
            route_query = route_query.filter(Column::UpdatedAt.gt(ts));
        }
        if let Some(ts) = query.updated_at_lt {
            route_query = route_query.filter(Column::UpdatedAt.lt(ts));
        }

        if let Some(sorts) = query.sorts {
            for sort in sorts {
                route_query = sort.apply_to_query(route_query);
            }
        } else {
            route_query = route_query.order_by_asc(Column::RoutePattern);
        }

        let page = query.page.unwrap_or(1);
        let total = route_query.clone().count(db).await?;

        let items = route_query
            .offset((page - 1) * Self::PER_PAGE)
            .limit(Self::PER_PAGE)
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
        known_routes_key: &str,
        blocked_routes_key: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let routes = Entity::find()
            .order_by_asc(Column::RoutePattern)
            .all(db)
            .await?;

        redis_pool.del::<(), _>(known_routes_key).await?;
        redis_pool.del::<(), _>(blocked_routes_key).await?;

        for route in routes {
            redis_pool
                .sadd::<(), _, _>(known_routes_key, route.route_pattern.clone())
                .await?;

            if route.is_blocked {
                redis_pool
                    .sadd::<(), _, _>(blocked_routes_key, route.route_pattern.clone())
                    .await?;
            }
        }

        Ok(())
    }
}
