use crate::error::{DbResult, ErrorCode, ErrorResponse};
use sea_orm::{entity::prelude::*, Condition, Order, QueryOrder, Set};

use super::*;
use crate::utils::color::{derive_text_color, DEFAULT_BG_COLOR};

impl Entity {
    pub const PER_PAGE: u64 = 20;

    async fn load_media_for_categories(
        conn: &DbConn,
        categories: Vec<Model>,
    ) -> DbResult<Vec<CategoryWithRelations>> {
        let mut media_ids = std::collections::HashSet::new();
        for cat in &categories {
            if let Some(id) = cat.cover_id {
                media_ids.insert(id);
            }
            if let Some(id) = cat.logo_id {
                media_ids.insert(id);
            }
        }

        let media_map = if !media_ids.is_empty() {
            super::super::media::Entity::find()
                .filter(
                    super::super::media::Column::Id
                        .is_in(media_ids.into_iter().collect::<Vec<i32>>()),
                )
                .all(conn)
                .await?
                .into_iter()
                .map(|m| {
                    (
                        m.id,
                        CategoryMedia {
                            id: m.id,
                            object_key: m.object_key,
                            file_url: m.file_url,
                            mime_type: m.mime_type,
                            width: m.width,
                            height: m.height,
                            size: m.size,
                        },
                    )
                })
                .collect::<std::collections::HashMap<i32, CategoryMedia>>()
        } else {
            std::collections::HashMap::new()
        };

        let results = categories
            .into_iter()
            .map(|cat| CategoryWithRelations {
                id: cat.id,
                name: cat.name,
                slug: cat.slug,
                parent_id: cat.parent_id,
                description: cat.description,
                color: cat.color,
                text_color: cat.text_color,
                is_active: cat.is_active,
                created_at: cat.created_at,
                updated_at: cat.updated_at,
                cover: cat.cover_id.and_then(|id| media_map.get(&id).cloned()),
                logo: cat.logo_id.and_then(|id| media_map.get(&id).cloned()),
            })
            .collect();

        Ok(results)
    }

    pub async fn create(conn: &DbConn, new_category: NewCategory) -> DbResult<Model> {
        let now = chrono::Utc::now().fixed_offset();
        let color = new_category
            .color
            .unwrap_or_else(|| DEFAULT_BG_COLOR.to_string());
        let text_color = derive_text_color(&color, new_category.text_color.as_deref());
        let is_active = new_category.is_active.unwrap_or(true);
        let category = ActiveModel {
            name: Set(new_category.name),
            slug: Set(new_category.slug),
            parent_id: Set(new_category.parent_id),
            description: Set(new_category.description),
            cover_id: Set(new_category.cover_id),
            logo_id: Set(new_category.logo_id),
            color: Set(color),
            text_color: Set(text_color),
            is_active: Set(is_active),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        match category.insert(conn).await {
            Ok(model) => Ok(model),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn update(
        conn: &DbConn,
        category_id: i32,
        update_category: UpdateCategory,
    ) -> DbResult<Option<Model>> {
        let category: Option<Model> = match Self::find_by_id(category_id).one(conn).await {
            Ok(category) => category,
            Err(err) => return Err(err.into()),
        };

        if let Some(category_model) = category {
            let mut category_active: ActiveModel = category_model.into();

            if let Some(name) = update_category.name {
                category_active.name = Set(name);
            }

            if let Some(slug) = update_category.slug {
                category_active.slug = Set(slug);
            }

            if let Some(parent_id) = update_category.parent_id {
                category_active.parent_id = Set(parent_id);
            }

            if let Some(description) = update_category.description {
                category_active.description = Set(description);
            }

            if let Some(cover_id) = update_category.cover_id {
                category_active.cover_id = Set(cover_id);
            }

            if let Some(logo_id) = update_category.logo_id {
                category_active.logo_id = Set(logo_id);
            }

            let mut recolor_dep: Option<String> = None;
            if let Some(color) = update_category.color {
                category_active.color = Set(color.clone());
                recolor_dep = Some(color);
            }

            if let Some(text_color) = update_category.text_color {
                category_active.text_color = Set(text_color);
            } else if let Some(color) = recolor_dep {
                category_active.text_color = Set(derive_text_color(&color, None));
            }

            if let Some(is_active) = update_category.is_active {
                category_active.is_active = Set(is_active);
            }

            category_active.updated_at = Set(chrono::Utc::now().fixed_offset());

            match category_active.update(conn).await {
                Ok(updated_category) => Ok(Some(updated_category)),
                Err(err) => Err(err.into()),
            }
        } else {
            Ok(None)
        }
    }

    pub async fn delete(conn: &DbConn, category_id: i32) -> DbResult<u64> {
        match Self::delete_by_id(category_id).exec(conn).await {
            Ok(result) => Ok(result.rows_affected),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn find_by_id_or_slug(
        conn: &DbConn,
        category_id: Option<i32>,
        category_slug: Option<String>,
    ) -> DbResult<Option<CategoryWithRelations>> {
        if category_id.is_none() && category_slug.is_none() {
            return Err(ErrorResponse::new(ErrorCode::InvalidInput)
                .with_message("Either category_id or category_slug must be provided"));
        }

        let mut query = Self::find();

        if let Some(id) = category_id {
            query = query.filter(Column::Id.eq(id));
        } else if let Some(slug) = category_slug {
            query = query.filter(Column::Slug.eq(slug));
        }

        let category = query.one(conn).await?;

        if let Some(cat) = category {
            let results = Self::load_media_for_categories(conn, vec![cat]).await?;
            Ok(results.into_iter().next())
        } else {
            Ok(None)
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

    pub async fn find_with_query(
        conn: &DbConn,
        query: CategoryQuery,
    ) -> DbResult<(Vec<CategoryWithRelations>, u64)> {
        let mut category_query = Self::find();

        if let Some(search_term) = query.search {
            let search_pattern = format!("%{}%", search_term.to_lowercase());
            category_query = category_query.filter(
                Condition::any()
                    .add(Column::Name.contains(&search_pattern))
                    .add(Column::Description.contains(&search_pattern)),
            );
        }

        if let Some(parent_id_filter) = query.parent_id {
            category_query = category_query.filter(Column::ParentId.eq(parent_id_filter));
        }

        if let Some(active) = query.is_active {
            category_query = category_query.filter(Column::IsActive.eq(active));
        }

        if let Some(ts) = query.created_at_gt {
            category_query = category_query.filter(Column::CreatedAt.gt(ts));
        }
        if let Some(ts) = query.created_at_lt {
            category_query = category_query.filter(Column::CreatedAt.lt(ts));
        }
        if let Some(ts) = query.updated_at_gt {
            category_query = category_query.filter(Column::UpdatedAt.gt(ts));
        }
        if let Some(ts) = query.updated_at_lt {
            category_query = category_query.filter(Column::UpdatedAt.lt(ts));
        }

        if let Some(sorts) = query.sorts {
            for sort in sorts {
                let column = match sort.field.as_str() {
                    "id" => Some(Column::Id),
                    "name" => Some(Column::Name),
                    "slug" => Some(Column::Slug),
                    "parent_id" => Some(Column::ParentId),
                    "description" => Some(Column::Description),
                    "cover_id" => Some(Column::CoverId),
                    "logo_id" => Some(Column::LogoId),
                    "color" => Some(Column::Color),
                    "text_color" => Some(Column::TextColor),
                    "is_active" => Some(Column::IsActive),
                    "created_at" => Some(Column::CreatedAt),
                    "updated_at" => Some(Column::UpdatedAt),
                    _ => None,
                };
                if let Some(col) = column {
                    category_query = category_query.order_by(col, sort.order);
                }
            }
        }

        let page = match query.page {
            Some(p) if p > 0 => p,
            _ => 1,
        };

        let paginator = category_query.paginate(conn, Self::PER_PAGE);

        let total = paginator.num_items().await?;
        let categories = paginator.fetch_page(page - 1).await?;

        let results = Self::load_media_for_categories(conn, categories).await?;

        Ok((results, total))
    }
}
