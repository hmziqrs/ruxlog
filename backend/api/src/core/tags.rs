use std::sync::Arc;

use tracing::{error, info, instrument};

use crate::{
    core::{
        context::CoreContext,
        types::{TagError, TagSummary},
    },
    db::sea_models::tag,
};

#[derive(Clone)]
pub struct TagService {
    core: Arc<CoreContext>,
}

impl TagService {
    pub fn new(core: Arc<CoreContext>) -> Self {
        Self { core }
    }

    #[instrument(skip(self))]
    pub async fn list_tags(&self) -> Result<Vec<TagSummary>, TagError> {
        let models = tag::Entity::find_all(&self.core.db)
            .await
            .map_err(|err| {
                error!(error = ?err, "Failed to load tags (core tags)");
                TagError::LoadFailed(err.to_string())
            })?;

        let results = models
            .into_iter()
            .map(|m| TagSummary {
                id: m.id,
                name: m.name,
                slug: m.slug,
                // Usage count can be wired to a real metric later.
                usage_count: None,
                created_at: m.created_at,
            })
            .collect::<Vec<_>>();

        info!(count = results.len(), "Loaded tags (core tags)");

        Ok(results)
    }
}

