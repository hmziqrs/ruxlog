use super::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Set,
};
use tower_sessions_redis_store::fred::interfaces::{HashesInterface, KeysInterface};

impl Entity {
    pub async fn find_by_key(db: &DatabaseConnection, key: &str) -> Result<Option<Model>, DbErr> {
        Entity::find().filter(Column::Key.eq(key)).one(db).await
    }

    pub async fn ensure_exists(
        db: &DatabaseConnection,
        key: &str,
        value: &str,
        value_type: Option<String>,
        is_sensitive: bool,
        source: &str,
    ) -> Result<Model, DbErr> {
        if let Some(existing) = Self::find_by_key(db, key).await? {
            return Ok(existing);
        }

        let active = ActiveModel {
            key: Set(key.to_string()),
            value: Set(value.to_string()),
            value_type: Set(value_type),
            is_sensitive: Set(is_sensitive),
            source: Set(source.to_string()),
            ..Default::default()
        };

        match active.insert(db).await {
            Ok(model) => Ok(model),
            Err(DbErr::Exec(exec_err)) => match Self::find_by_key(db, key).await? {
                Some(existing) => Ok(existing),
                None => Err(DbErr::Exec(exec_err)),
            },
            Err(err) => Err(err),
        }
    }

    pub async fn upsert_value(
        db: &DatabaseConnection,
        key: &str,
        value: &str,
        value_type: Option<String>,
        description: Option<String>,
        is_sensitive: bool,
        source: &str,
        updated_by: Option<i32>,
    ) -> Result<Model, DbErr> {
        if let Some(existing) = Self::find_by_key(db, key).await? {
            let mut active: ActiveModel = existing.into();
            active.value = Set(value.to_string());
            active.value_type = Set(value_type);
            active.description = Set(description);
            active.is_sensitive = Set(is_sensitive);
            active.source = Set(source.to_string());
            active.updated_by = Set(updated_by);
            active.updated_at = Set(chrono::Utc::now().fixed_offset());
            active.update(db).await
        } else {
            let active = ActiveModel {
                key: Set(key.to_string()),
                value: Set(value.to_string()),
                value_type: Set(value_type),
                description: Set(description),
                is_sensitive: Set(is_sensitive),
                source: Set(source.to_string()),
                updated_by: Set(updated_by),
                ..Default::default()
            };
            active.insert(db).await
        }
    }

    pub async fn delete_by_key(db: &DatabaseConnection, key: &str) -> Result<u64, DbErr> {
        let res = Entity::delete_many()
            .filter(Column::Key.eq(key))
            .exec(db)
            .await?;
        Ok(res.rows_affected)
    }

    pub async fn list(
        db: &DatabaseConnection,
        page: u64,
        per_page: u64,
        search: Option<String>,
        is_sensitive: Option<bool>,
        value_type: Option<String>,
    ) -> Result<(Vec<Model>, u64), DbErr> {
        let mut query = Entity::find();

        if let Some(search) = search {
            let like = format!("%{}%", search);
            let cond = Condition::any()
                .add(Column::Key.contains(&like))
                .add(Column::Description.contains(&like));
            query = query.filter(cond);
        }

        if let Some(sensitive) = is_sensitive {
            query = query.filter(Column::IsSensitive.eq(sensitive));
        }

        if let Some(vt) = value_type {
            query = query.filter(Column::ValueType.eq(vt));
        }

        let total = query.clone().count(db).await?;
        let items = query
            .order_by_asc(Column::Key)
            .offset((page.saturating_sub(1)) * per_page)
            .limit(per_page)
            .all(db)
            .await?;
        Ok((items, total))
    }

    pub async fn sync_all_to_redis(
        db: &DatabaseConnection,
        redis_pool: tower_sessions_redis_store::fred::prelude::Pool,
        value_hash: &str,
        meta_hash: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let all = Entity::find().order_by_asc(Column::Key).all(db).await?;

        redis_pool.del::<(), _>(value_hash).await?;
        redis_pool.del::<(), _>(meta_hash).await?;

        for item in all {
            redis_pool
                .hset::<(), _, _>(value_hash, vec![(&item.key, &item.value)])
                .await?;
            let meta = serde_json::json!({
                "value_type": item.value_type,
                "is_sensitive": item.is_sensitive,
                "updated_at": item.updated_at,
            })
            .to_string();
            redis_pool
                .hset::<(), _, _>(meta_hash, vec![(&item.key, meta)])
                .await?;
        }
        Ok(())
    }
}
