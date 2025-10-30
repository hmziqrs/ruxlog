use sea_orm::{ActiveModelTrait, Set};

use crate::error::{DbResult, ErrorResponse};

use super::{model::ActiveModel, Entity, Model, NewMediaVariant};

impl Entity {
    pub async fn create_many(
        conn: &sea_orm::DatabaseConnection,
        variants: Vec<NewMediaVariant>,
    ) -> DbResult<Vec<Model>> {
        if variants.is_empty() {
            return Ok(Vec::new());
        }

        let active_models: Vec<ActiveModel> = variants
            .into_iter()
            .map(|variant| ActiveModel {
                media_id: Set(variant.media_id),
                object_key: Set(variant.object_key),
                mime_type: Set(variant.mime_type),
                width: Set(variant.width),
                height: Set(variant.height),
                size: Set(variant.size),
                extension: Set(variant.extension),
                quality: Set(variant.quality),
                variant_type: Set(variant.variant_type),
                ..Default::default()
            })
            .collect();

        let mut inserted = Vec::with_capacity(active_models.len());
        for model in active_models {
            inserted.push(model.insert(conn).await.map_err(ErrorResponse::from)?);
        }

        Ok(inserted)
    }
}
