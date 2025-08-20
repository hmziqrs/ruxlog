use crate::error::{DbResult, ErrorCode, ErrorResponse};
use sea_orm::{entity::prelude::*, Condition, Order, QueryOrder, Set};

use super::*;
use crate::utils::color::{derive_text_color, DEFAULT_BG_COLOR};

impl Entity {
    pub const PER_PAGE: u64 = 20;

    pub async fn create(conn: &DbConn, new_tag: NewTag) -> DbResult<Model> {
        let now = chrono::Utc::now().fixed_offset();
        let color = new_tag
            .color
            .unwrap_or_else(|| DEFAULT_BG_COLOR.to_string());
        let text_color = derive_text_color(&color, new_tag.text_color.as_deref());
        let is_active = new_tag.is_active.unwrap_or(true);
        let tag = ActiveModel {
            name: Set(new_tag.name),
            slug: Set(new_tag.slug),
            description: Set(new_tag.description),
            color: Set(color),
            text_color: Set(text_color),
            is_active: Set(is_active),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        match tag.insert(conn).await {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn update(
        conn: &DbConn,
        tag_id: i32,
        update_tag: UpdateTag,
    ) -> DbResult<Option<Model>> {
        let tag: Option<Model> = match Self::find_by_id(tag_id).one(conn).await {
            Ok(tag) => tag,
            Err(err) => return Err(err.into()),
        };

        if let Some(tag_model) = tag {
            let mut tag_active: ActiveModel = tag_model.into();

            if let Some(name) = update_tag.name {
                tag_active.name = Set(name);
            }

            if let Some(slug) = update_tag.slug {
                tag_active.slug = Set(slug);
            }

            if let Some(description) = update_tag.description {
                tag_active.description = Set(Some(description));
            }

            let mut recolor_dep: Option<String> = None;
            if let Some(color) = update_tag.color {
                tag_active.color = Set(color.clone());
                recolor_dep = Some(color);
            }

            if let Some(text_color) = update_tag.text_color {
                tag_active.text_color = Set(text_color);
            } else if let Some(color) = recolor_dep {
                tag_active.text_color = Set(derive_text_color(&color, None));
            }

            if let Some(is_active) = update_tag.is_active {
                tag_active.is_active = Set(is_active);
            }

            tag_active.updated_at = Set(chrono::Utc::now().fixed_offset());

            match tag_active.update(conn).await {
                Ok(updated_tag) => Ok(Some(updated_tag)),
                Err(err) => Err(err.into()),
            }
        } else {
            Ok(None)
        }
    }

    pub async fn delete(conn: &DbConn, tag_id: i32) -> DbResult<u64> {
        match Self::delete_by_id(tag_id).exec(conn).await {
            Ok(result) => Ok(result.rows_affected),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn get_by_id(conn: &DbConn, tag_id: i32) -> DbResult<Option<Model>> {
        match Self::find_by_id(tag_id).one(conn).await {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn find_by_id_with_404(conn: &DbConn, tag_id: i32) -> DbResult<Model> {
        match Self::find_by_id(tag_id).one(conn).await {
            Ok(Some(model)) => Ok(model),
            Ok(None) => Err(ErrorResponse::new(ErrorCode::RecordNotFound)
                .with_message(&format!("Tag with ID {} not found", tag_id))),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn find_all(conn: &DbConn) -> DbResult<Vec<Model>> {
        match Self::find()
            .order_by(Column::Name, Order::Desc)
            .all(conn)
            .await
        {
            Ok(models) => Ok(models),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn find_with_query(conn: &DbConn, query: TagQuery) -> DbResult<(Vec<Model>, u64)> {
        let mut tag_query = Self::find();

        if let Some(search_term) = query.search {
            let search_pattern = format!("%{}%", search_term.to_lowercase());
            tag_query = tag_query.filter(
                Condition::any()
                    .add(Column::Name.contains(&search_pattern))
                    .add(Column::Description.contains(&search_pattern)),
            );
        }

        // Optional is_active filter
        if let Some(active) = query.is_active {
            tag_query = tag_query.filter(Column::IsActive.eq(active));
        }

        // Sorting: prefer dynamic multi-field sorts if provided, else default to name desc
        if let Some(sorts) = &query.sorts {
            if !sorts.is_empty() {
                for s in sorts {
                    // Map string field names to columns; unknown fields are ignored
                    let column = match s.field.as_str() {
                        "id" => Some(Column::Id),
                        "name" => Some(Column::Name),
                        "slug" => Some(Column::Slug),
                        "description" => Some(Column::Description),
                        "color" => Some(Column::Color),
                        "text_color" => Some(Column::TextColor),
                        "is_active" => Some(Column::IsActive),
                        "created_at" => Some(Column::CreatedAt),
                        "updated_at" => Some(Column::UpdatedAt),
                        _ => None,
                    };

                    if let Some(col) = column {
                        let ord = s.order.clone();
                        tag_query = tag_query.order_by(col, ord);
                    }
                }
            }
        }

        let page = match query.page {
            Some(p) if p > 0 => p,
            _ => 1,
        };
        let paginator = tag_query.paginate(conn, Self::PER_PAGE);

        match paginator.num_items().await {
            Ok(total) => match paginator.fetch_page(page - 1).await {
                Ok(results) => Ok((results, total)),
                Err(err) => Err(err.into()),
            },
            Err(err) => Err(err.into()),
        }
    }
}
