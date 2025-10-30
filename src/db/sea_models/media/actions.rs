use crate::error::{DbResult, ErrorResponse};
use sea_orm::{entity::prelude::*, Condition, Order, QueryOrder, Set};

use super::{
    model::{ActiveModel, Column, Entity},
    MediaQuery, MediaReference, Model, NewMedia,
};

impl Entity {
    pub const PER_PAGE: u64 = 20;
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
            content_hash: Set(payload.content_hash),
            is_optimized: Set(payload.is_optimized),
            optimized_at: Set(payload.optimized_at),
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

    pub async fn find_by_hash(conn: &DbConn, hash: &str) -> DbResult<Option<Model>> {
        <Self as EntityTrait>::find()
            .filter(Column::ContentHash.eq(hash))
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

    pub async fn find_with_query(conn: &DbConn, query: MediaQuery) -> DbResult<(Vec<Model>, u64)> {
        let mut media_query = Self::find();

        if let Some(search_term) = query.search {
            let search_pattern = format!("%{}%", search_term.to_lowercase());
            media_query = media_query.filter(
                Condition::any()
                    .add(Column::ObjectKey.contains(&search_pattern))
                    .add(Column::FileUrl.contains(&search_pattern))
                    .add(Column::MimeType.contains(&search_pattern))
                    .add(Column::Extension.contains(&search_pattern)),
            );
        }

        if let Some(reference) = query.reference_type {
            media_query = media_query.filter(Column::ReferenceType.eq(reference));
        }

        if let Some(uploader_id) = query.uploader_id {
            media_query = media_query.filter(Column::UploaderId.eq(uploader_id));
        }

        if let Some(mime) = query.mime_type {
            let pattern = format!("%{}%", mime.to_lowercase());
            media_query = media_query.filter(Column::MimeType.contains(&pattern));
        }

        if let Some(ext) = query.extension {
            let pattern = format!("%{}%", ext.to_lowercase());
            media_query = media_query.filter(Column::Extension.contains(&pattern));
        }

        if let Some(ts) = query.created_at_gt {
            media_query = media_query.filter(Column::CreatedAt.gt(ts));
        }
        if let Some(ts) = query.created_at_lt {
            media_query = media_query.filter(Column::CreatedAt.lt(ts));
        }
        if let Some(ts) = query.updated_at_gt {
            media_query = media_query.filter(Column::UpdatedAt.gt(ts));
        }
        if let Some(ts) = query.updated_at_lt {
            media_query = media_query.filter(Column::UpdatedAt.lt(ts));
        }

        // Sorting: support multiple field sorts; default to created_at desc
        if let Some(sorts) = &query.sorts {
            if !sorts.is_empty() {
                for s in sorts {
                    let column = match s.field.as_str() {
                        "id" => Some(Column::Id),
                        "object_key" => Some(Column::ObjectKey),
                        "file_url" => Some(Column::FileUrl),
                        "mime_type" => Some(Column::MimeType),
                        "width" => Some(Column::Width),
                        "height" => Some(Column::Height),
                        "size" => Some(Column::Size),
                        "extension" => Some(Column::Extension),
                        "uploader_id" => Some(Column::UploaderId),
                        "reference_type" => Some(Column::ReferenceType),
                        "created_at" => Some(Column::CreatedAt),
                        "updated_at" => Some(Column::UpdatedAt),
                        _ => None,
                    };

                    if let Some(col) = column {
                        let ord: Order = s.order.clone();
                        media_query = media_query.order_by(col, ord);
                    }
                }
            } else {
                media_query = media_query.order_by_desc(Column::CreatedAt);
            }
        } else {
            media_query = media_query.order_by_desc(Column::CreatedAt);
        }

        let page = match query.page {
            Some(p) if p > 0 => p,
            _ => 1,
        };
        let paginator = media_query.paginate(conn, Self::PER_PAGE);

        match paginator.num_items().await {
            Ok(total) => match paginator.fetch_page(page - 1).await {
                Ok(results) => Ok((results, total)),
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        }
    }
}
