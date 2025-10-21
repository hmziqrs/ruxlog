use crate::error::{DbResult, ErrorResponse};
use sea_orm::{entity::prelude::*, QueryOrder, Set};

use super::{
    model::{ActiveModel, Column, Entity},
    MediaReference, Model, NewMedia,
};

impl Entity {
    pub async fn create(conn: &DbConn, payload: NewMedia) -> DbResult<Model> {
        let now = chrono::Utc::now().fixed_offset();
        let media = ActiveModel {
            object_key: Set(payload.object_key),
            file_url: Set(payload.file_url),
            mime_type: Set(payload.mime_type),
            width: Set(payload.width),
            height: Set(payload.height),
            size: Set(payload.size),
            extension: Set(payload.extension),
            uploader_id: Set(payload.uploader_id),
            reference_type: Set(payload.reference_type),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        media.insert(conn).await.map_err(Into::into)
    }

    pub async fn find_by_id(conn: &DbConn, id: i32) -> DbResult<Option<Model>> {
        <Self as EntityTrait>::find_by_id(id)
            .one(conn)
            .await
            .map_err(ErrorResponse::from)
    }

    pub async fn delete_by_id(conn: &DbConn, id: i32) -> DbResult<Option<Model>> {
        match <Self as EntityTrait>::find_by_id(id)
            .one(conn)
            .await
            .map_err(ErrorResponse::from)?
        {
            Some(model) => {
                let active_model: ActiveModel = model.clone().into();
                active_model
                    .delete(conn)
                    .await
                    .map_err(ErrorResponse::from)?;
                Ok(Some(model))
            }
            None => Ok(None),
        }
    }

    pub async fn list_by_uploader(conn: &DbConn, uploader_id: i32) -> DbResult<Vec<Model>> {
        Self::find()
            .filter(Column::UploaderId.eq(uploader_id))
            .order_by_desc(Column::CreatedAt)
            .all(conn)
            .await
            .map_err(ErrorResponse::from)
    }

    pub async fn list_by_reference(
        conn: &DbConn,
        reference: MediaReference,
    ) -> DbResult<Vec<Model>> {
        Self::find()
            .filter(Column::ReferenceType.eq(reference))
            .order_by_desc(Column::CreatedAt)
            .all(conn)
            .await
            .map_err(ErrorResponse::from)
    }
}
