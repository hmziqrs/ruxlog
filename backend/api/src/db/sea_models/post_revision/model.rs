use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "post_revisions")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,

    pub post_id: i32,

    // Stored as a JSON string (EditorJS blocks). Sanitized on read through the
    // ammonia allowlist — the same control PostWithRelations::content uses — so
    // every client-facing serialization (autosave result, revisions list) is
    // XSS-clean regardless of which handler produced it. Stored value is left
    // untouched. See utils/sanitize.rs (plan Phase 6e).
    #[serde(serialize_with = "crate::utils::sanitize::serialize_sanitized_content_string")]
    pub content: String,

    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub metadata: Option<serde_json::Value>,

    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::super::post::Entity",
        from = "Column::PostId",
        to = "super::super::post::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Post,
}

impl Related<super::super::post::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Post.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {
    use super::*;

    // End-to-end proof that the field-level serialize_with fires when a Model
    // is serialized (the path the autosave + revisions-list endpoints take via
    // serde_json::json! / Json). A malicious EditorJS payload stored in the
    // revision's content string must come back XSS-clean, with the stored value
    // untouched. (Plan Phase 6e; mirrors the PostWithRelations control.)
    #[test]
    fn serialized_revision_content_is_xss_clean() {
        let stored = serde_json::to_string(&serde_json::json!({
            "blocks": [
                { "type": "paragraph", "data": { "text": "<script>alert(1)</script>hi" } },
                { "type": "raw", "data": { "html": "<a href=javascript:alert(1)>x</a>" } }
            ]
        }))
        .unwrap();

        let model = Model {
            id: 1,
            post_id: 1,
            content: stored.clone(),
            metadata: None,
            created_at: chrono::Utc::now().fixed_offset(),
        };

        // serde_json::json! / Json both route through Serialize, i.e. the
        // field's serialize_with — the same code path as the controllers.
        let serialized = serde_json::to_value(&model).unwrap();
        let emitted = serialized["content"].as_str().unwrap();

        assert!(
            !emitted.contains("<script"),
            "script tag must be stripped from serialized revision: {emitted}"
        );
        assert!(
            !emitted.contains("javascript:"),
            "javascript: URL must be stripped: {emitted}"
        );
        assert!(emitted.contains("hi"), "benign content survives: {emitted}");

        // The stored value is untouched (sanitization-on-read).
        assert!(
            stored.contains("<script>"),
            "stored content must not be mutated: {stored}"
        );
    }
}
