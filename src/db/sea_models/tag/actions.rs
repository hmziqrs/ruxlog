use sea_orm::{entity::prelude::*, Condition, Order, QueryOrder, Set};

use super::*;

impl Entity {
    const PER_PAGE: u64 = 20;

    // Create a new tag
    pub async fn create(conn: &DbConn, new_tag: NewTag) -> Result<Model, DbErr> {
        let now = chrono::Utc::now().naive_utc();
        let tag = ActiveModel {
            name: Set(new_tag.name),
            slug: Set(new_tag.slug),
            description: Set(new_tag.description),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        tag.insert(conn).await
    }

    // Update an existing tag
    pub async fn update(
        conn: &DbConn,
        tag_id: i32,
        update_tag: UpdateTag,
    ) -> Result<Option<Model>, DbErr> {
        let tag: Option<Model> = Entity::get_by_id(conn, tag_id).await?;

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

            tag_active.updated_at = Set(chrono::Utc::now().naive_utc());

            let updated_tag = tag_active.update(conn).await?;
            Ok(Some(updated_tag))
        } else {
            Ok(None)
        }
    }

    // Delete a tag
    pub async fn delete(conn: &DbConn, tag_id: i32) -> Result<u64, DbErr> {
        let result = Entity::delete_by_id(tag_id).exec(conn).await?;
        Ok(result.rows_affected)
    }

    // Find tag by ID
    pub async fn get_by_id(conn: &DbConn, tag_id: i32) -> Result<Option<Model>, DbErr> {
        Entity::find_by_id(tag_id).one(conn).await
    }

    // Find all tags
    pub async fn find_all(conn: &DbConn) -> Result<Vec<Model>, DbErr> {
        Entity::find()
            .order_by(Column::Name, Order::Desc)
            .all(conn)
            .await
    }

    // Find tags with query parameters
    pub async fn find_with_query(
        conn: &DbConn,
        query: TagQuery,
    ) -> Result<(Vec<Model>, u64), DbErr> {
        let mut tag_query = Entity::find();

        // Handle search filter
        if let Some(search_term) = query.search {
            let search_pattern = format!("%{}%", search_term.to_lowercase());
            tag_query = tag_query.filter(
                Condition::any()
                    .add(Column::Name.contains(&search_pattern))
                    .add(Column::Description.contains(&search_pattern)),
            );
        }

        // Handle ordering
        match query.sort_order.as_deref() {
            Some("asc") => {
                tag_query = tag_query.order_by(Column::Name, Order::Asc);
            }
            _ => {
                tag_query = tag_query.order_by(Column::Name, Order::Desc);
            }
        }

        // Handle pagination
        let page = match query.page_no {
            Some(p) if p > 0 => p,
            _ => 1,
        };
        let paginator = tag_query.paginate(conn, Self::PER_PAGE);
        let total = paginator.num_items().await?;
        let results = paginator.fetch_page(page - 1).await?;

        Ok((results, total))
    }
}
