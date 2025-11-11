use sea_orm::prelude::*;
use sea_orm::{ActiveModelTrait, ConnectionTrait, Set};

use super::model::{ActiveModel, Column, Entity, EntityType, Model};
use crate::error::DbResult;
use sea_orm::{PaginatorTrait, QueryOrder};

impl Entity {
    pub const PER_PAGE: u64 = 20;

    /// Track media usage for an entity field
    pub async fn track_usage<C>(
        conn: &C,
        media_id: i32,
        entity_type: EntityType,
        entity_id: i32,
        field_name: &str,
    ) -> DbResult<Model>
    where
        C: ConnectionTrait,
    {
        let now = chrono::Utc::now().fixed_offset();

        let usage = ActiveModel {
            media_id: Set(media_id),
            entity_type: Set(entity_type),
            entity_id: Set(entity_id),
            field_name: Set(field_name.to_string()),
            created_at: Set(now),
            ..Default::default()
        };

        match usage.insert(conn).await {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    /// Remove media usage for an entity field
    pub async fn untrack_usage<C>(
        conn: &C,
        entity_type: EntityType,
        entity_id: i32,
        field_name: &str,
    ) -> DbResult<u64>
    where
        C: ConnectionTrait,
    {
        match Self::delete_many()
            .filter(Column::EntityType.eq(entity_type))
            .filter(Column::EntityId.eq(entity_id))
            .filter(Column::FieldName.eq(field_name))
            .exec(conn)
            .await
        {
            Ok(result) => Ok(result.rows_affected),
            Err(err) => Err(err.into()),
        }
    }

    /// Update media usage - removes old and tracks new
    pub async fn update_usage<C>(
        conn: &C,
        _old_media_id: Option<i32>,
        new_media_id: Option<i32>,
        entity_type: EntityType,
        entity_id: i32,
        field_name: &str,
    ) -> DbResult<()>
    where
        C: ConnectionTrait,
    {
        // Remove old usage
        Self::untrack_usage(conn, entity_type, entity_id, field_name).await?;

        // Track new usage if provided
        if let Some(mid) = new_media_id {
            Self::track_usage(conn, mid, entity_type, entity_id, field_name).await?;
        }

        Ok(())
    }

    /// Get all usages for a specific media
    pub async fn find_by_media_id<C>(conn: &C, media_id: i32) -> DbResult<Vec<Model>>
    where
        C: ConnectionTrait,
    {
        match Self::find()
            .filter(Column::MediaId.eq(media_id))
            .all(conn)
            .await
        {
            Ok(usages) => Ok(usages),
            Err(err) => Err(err.into()),
        }
    }

    /// Remove all usages for an entity
    pub async fn delete_by_entity<C>(
        conn: &C,
        entity_type: EntityType,
        entity_id: i32,
    ) -> DbResult<u64>
    where
        C: ConnectionTrait,
    {
        match Self::delete_many()
            .filter(Column::EntityType.eq(entity_type))
            .filter(Column::EntityId.eq(entity_id))
            .exec(conn)
            .await
        {
            Ok(result) => Ok(result.rows_affected),
            Err(err) => Err(err.into()),
        }
    }

    /// Find media usage with filtering and pagination
    pub async fn find_with_filters(
        conn: &DbConn,
        media_id: Option<i32>,
        entity_type: Option<String>,
        entity_id: Option<i32>,
        field_name: Option<String>,
        page: Option<u64>,
    ) -> DbResult<(Vec<Model>, u64)> {
        let mut query = Self::find();

        if let Some(mid) = media_id {
            query = query.filter(Column::MediaId.eq(mid));
        }

        if let Some(et) = entity_type {
            if let Ok(entity_type_enum) = EntityType::from_str(&et.to_lowercase()) {
                query = query.filter(Column::EntityType.eq(entity_type_enum));
            }
        }

        if let Some(eid) = entity_id {
            query = query.filter(Column::EntityId.eq(eid));
        }

        if let Some(fn_name) = field_name {
            let pattern = format!("%{}%", fn_name.to_lowercase());
            query = query.filter(Column::FieldName.contains(&pattern));
        }

        query = query.order_by_desc(Column::CreatedAt);

        let page = match page {
            Some(p) if p > 0 => p,
            _ => 1,
        };

        let paginator = query.paginate(conn, Self::PER_PAGE);

        match paginator.num_items().await {
            Ok(total) => match paginator.fetch_page(page - 1).await {
                Ok(results) => Ok((results, total)),
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        }
    }
}
