use sea_orm::prelude::*;
use sea_orm::{ActiveModelTrait, ConnectionTrait, Set};

use super::model::{ActiveModel, Column, Entity, EntityType, Model};
use crate::error::DbResult;

impl Entity {
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
}
